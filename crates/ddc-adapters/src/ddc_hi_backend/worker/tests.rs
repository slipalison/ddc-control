use std::fmt;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use ddc_core::domain::{DdcError, MonitorId, VcpCode, VcpValue};
use ddc_core::ports::MonitorBackend;

use super::super::DdcHiBudgets;
use super::super::identity::DisplayIdentity;
use super::super::retry::{Clock, RetryPolicy, SystemClock};
use super::{DdcHandle, DisplaySource, HandleError, TransactError, Worker, WorkerClient};

/// Holds a transaction until the test opens it.
#[derive(Debug, Clone, Default)]
struct Gate(Arc<(Mutex<bool>, Condvar)>);

impl Gate {
    fn wait(&self) {
        let (open, changed) = &*self.0;
        let mut open = lock(open);
        while !*open {
            open = changed.wait(open).unwrap_or_else(PoisonError::into_inner);
        }
    }

    fn open(&self) {
        let (open, changed) = &*self.0;
        *lock(open) = true;
        changed.notify_all();
    }
}

/// How a fake display answers every transaction.
#[derive(Debug, Clone)]
enum Behaviour {
    Answer,
    Fail(&'static str),
    /// Fails this many more times with [`FLAKY`], then answers.
    Flaky(u32),
    /// Fails and is unplugged, like a monitor pulled mid-transaction.
    Vanish,
    Block(Gate),
    Crash,
}

const FLAKY: &str = "flaky i2c bus";

#[derive(Debug, Clone)]
struct FakeDisplay {
    name: &'static str,
    behaviour: Behaviour,
}

impl FakeDisplay {
    fn new(name: &'static str, behaviour: Behaviour) -> Self {
        Self { name, behaviour }
    }

    /// How this transaction goes; a flaky display spends one failure.
    fn take_turn(&mut self) -> Behaviour {
        match &mut self.behaviour {
            Behaviour::Flaky(0) => Behaviour::Answer,
            Behaviour::Flaky(left) => {
                *left -= 1;
                Behaviour::Fail(FLAKY)
            }
            other => other.clone(),
        }
    }
}

/// A transaction that reached a fake display.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Call {
    Capabilities(&'static str),
    Read(&'static str, VcpCode),
    Write(&'static str, VcpCode, u16),
}

#[derive(Debug, Default)]
struct Bus {
    displays: Vec<FakeDisplay>,
    calls: Vec<Call>,
    enumerations: usize,
}

/// Fake [`DisplaySource`]; clones share the same bus, so a test keeps one to
/// replug displays and inspect calls while the worker owns another.
#[derive(Debug, Clone, Default)]
struct FakeDisplays(Arc<Mutex<Bus>>);

impl FakeDisplays {
    fn with(displays: impl IntoIterator<Item = FakeDisplay>) -> Self {
        let source = Self::default();
        source.plug(displays);
        source
    }

    fn plug(&self, displays: impl IntoIterator<Item = FakeDisplay>) {
        lock(&self.0).displays = displays.into_iter().collect();
    }

    fn calls(&self) -> Vec<Call> {
        lock(&self.0).calls.clone()
    }

    fn enumerations(&self) -> usize {
        lock(&self.0).enumerations
    }
}

impl DisplaySource for FakeDisplays {
    type Handle = FakeHandle;

    fn enumerate(&mut self) -> Vec<(DisplayIdentity, FakeHandle)> {
        let mut bus = lock(&self.0);
        bus.enumerations += 1;
        bus.displays
            .iter()
            .map(|display| {
                let identity = DisplayIdentity {
                    description: display.name.to_owned(),
                    edid: None,
                };
                let handle = FakeHandle {
                    bus: Arc::clone(&self.0),
                    name: display.name,
                };
                (identity, handle)
            })
            .collect()
    }
}

/// Fake displays whose enumeration takes `cost` on `clock`, like the
/// ~1.1 s EDID scan of `ddc-hi` on the dev machine.
#[derive(Debug)]
struct SlowDisplays<C> {
    displays: FakeDisplays,
    clock: C,
    cost: Duration,
}

impl<C: Clock + Send + 'static> DisplaySource for SlowDisplays<C> {
    type Handle = FakeHandle;

    fn enumerate(&mut self) -> Vec<(DisplayIdentity, FakeHandle)> {
        self.clock.sleep(self.cost);
        self.displays.enumerate()
    }
}

#[derive(Debug)]
struct FakeHandle {
    bus: Arc<Mutex<Bus>>,
    name: &'static str,
}

impl FakeHandle {
    /// Logs `call`, then answers as the display's behaviour says. A display
    /// unplugged since enumeration fails like a dead bus.
    fn transact<T>(&self, call: Call, answer: T) -> Result<T, HandleError> {
        match self.turn(call) {
            None => Err(HandleError::new("no such device")),
            Some(Behaviour::Answer | Behaviour::Flaky(_)) => Ok(answer),
            Some(Behaviour::Fail(message)) => Err(HandleError::new(message)),
            Some(Behaviour::Vanish) => {
                lock(&self.bus)
                    .displays
                    .retain(|display| display.name != self.name);
                Err(HandleError::new("no such device"))
            }
            Some(Behaviour::Block(gate)) => {
                gate.wait();
                Ok(answer)
            }
            Some(Behaviour::Crash) => crash(),
        }
    }

    fn turn(&self, call: Call) -> Option<Behaviour> {
        let mut bus = lock(&self.bus);
        bus.calls.push(call);
        bus.displays
            .iter_mut()
            .find(|display| display.name == self.name)
            .map(FakeDisplay::take_turn)
    }
}

// reason: simulates a bug inside the transport, which unwinds the worker.
#[allow(clippy::panic)]
fn crash() -> ! {
    panic!("scripted transport bug")
}

const FAKE_CAPABILITIES: &[u8] = b"(prot(monitor)vcp(10 12))\0";
const FAKE_VALUE: VcpValue = VcpValue {
    current: 50,
    max: 100,
};

impl DdcHandle for FakeHandle {
    fn read_capabilities(&mut self) -> Result<Vec<u8>, HandleError> {
        self.transact(Call::Capabilities(self.name), FAKE_CAPABILITIES.to_vec())
    }

    fn read_vcp(&mut self, code: VcpCode) -> Result<VcpValue, HandleError> {
        self.transact(Call::Read(self.name, code), FAKE_VALUE)
    }

    fn write_vcp(&mut self, code: VcpCode, value: u16) -> Result<(), HandleError> {
        self.transact(Call::Write(self.name, code, value), ())
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn budgets(vcp: Duration) -> DdcHiBudgets {
    DdcHiBudgets {
        vcp,
        capabilities: vcp,
        enumerate: Duration::from_secs(5),
    }
}

/// Retries without pauses, so no test sleeps between attempts.
fn no_backoff() -> RetryPolicy {
    RetryPolicy {
        backoff: Duration::ZERO,
        ..RetryPolicy::default()
    }
}

/// A worker on its own thread.
fn spawn(source: &FakeDisplays, budgets: DdcHiBudgets) -> WorkerClient<FakeDisplays> {
    WorkerClient::spawn(source.clone(), budgets, no_backoff()).unwrap()
}

/// Time that only moves when the worker sleeps.
#[derive(Debug, Clone)]
struct VirtualClock(Arc<Mutex<(Instant, Vec<Duration>)>>);

impl VirtualClock {
    fn new() -> Self {
        Self(Arc::new(Mutex::new((Instant::now(), Vec::new()))))
    }

    fn sleeps(&self) -> Vec<Duration> {
        lock(&self.0).1.clone()
    }
}

impl Clock for VirtualClock {
    fn now(&self) -> Instant {
        lock(&self.0).0
    }

    fn sleep(&self, duration: Duration) {
        let mut time = lock(&self.0);
        time.0 += duration;
        time.1.push(duration);
    }
}

/// What a single read did on a worker run in the test thread, after one
/// instant enumeration, with the default retry policy and virtual time.
#[derive(Debug)]
struct ReadRun {
    result: Result<VcpValue, DdcError>,
    reads: usize,
    sleeps: Vec<Duration>,
}

fn read_on_worker(behaviour: Behaviour, budget: Duration) -> ReadRun {
    let source = FakeDisplays::with([FakeDisplay::new("a", behaviour)]);
    let clock = VirtualClock::new();
    let mut worker = Worker::new(source.clone(), RetryPolicy::default(), clock.clone());
    worker.enumerate();

    let result = worker
        .read_vcp(&id("a"), VcpCode::BRIGHTNESS, clock.now() + budget)
        .map_err(|error| error.for_caller(&id("a")));

    ReadRun {
        result,
        reads: source.calls().len(),
        sleeps: clock.sleeps(),
    }
}

/// Virtual time one enumeration takes: the ~1.1 s measured on the dev
/// machine (D-7).
const ENUMERATION: Duration = Duration::from_millis(1100);

type SlowWorker = Worker<SlowDisplays<VirtualClock>, VirtualClock>;

/// What one transaction on display "a" did on a worker run in the test
/// thread, after an enumeration that took [`ENUMERATION`].
#[derive(Debug)]
struct SlowRun<T> {
    result: Result<T, DdcError>,
    enumerations: usize,
    took: Duration,
}

fn on_slow_worker<T>(
    behaviour: Behaviour,
    budget: Duration,
    op: impl FnOnce(&mut SlowWorker, &MonitorId, Instant) -> Result<T, TransactError>,
) -> SlowRun<T> {
    let source = FakeDisplays::with([FakeDisplay::new("a", behaviour)]);
    let clock = VirtualClock::new();
    let displays = SlowDisplays {
        displays: source.clone(),
        clock: clock.clone(),
        cost: ENUMERATION,
    };
    let mut worker = Worker::new(displays, RetryPolicy::default(), clock.clone());
    worker.enumerate();
    let started = clock.now();

    let result =
        op(&mut worker, &id("a"), started + budget).map_err(|error| error.for_caller(&id("a")));

    SlowRun {
        result,
        enumerations: source.enumerations(),
        took: clock.now() - started,
    }
}

fn read_brightness(
    worker: &mut SlowWorker,
    id: &MonitorId,
    deadline: Instant,
) -> Result<VcpValue, TransactError> {
    worker.read_vcp(id, VcpCode::BRIGHTNESS, deadline)
}

fn assert_transport<T: fmt::Debug>(result: &Result<T, DdcError>, containing: &str) {
    assert!(
        matches!(result, Err(DdcError::Transport(message)) if message.contains(containing)),
        "{result:?}"
    );
}

fn id(name: &str) -> MonitorId {
    MonitorId::new(name)
}

#[test]
fn caller_receives_timeout_when_worker_does_not_answer_within_budget() {
    let gate = Gate::default();
    let source = FakeDisplays::with([FakeDisplay::new("stuck", Behaviour::Block(gate.clone()))]);
    let client = spawn(&source, budgets(Duration::from_millis(20)));
    let started = Instant::now();

    let result = client.read_vcp(&id("stuck"), VcpCode::BRIGHTNESS);

    let waited = started.elapsed();
    gate.open();
    assert_eq!(result, Err(DdcError::Timeout));
    assert!(waited < Duration::from_secs(1), "waited {waited:?}");
}

#[test]
fn maps_backend_failures_to_transport_or_timeout_without_leaking_ddc_hi_errors() {
    const NAK: &str = "i2c nak from slave 0x37";
    let gate = Gate::default();
    let source = FakeDisplays::with([
        FakeDisplay::new("mute", Behaviour::Fail(NAK)),
        FakeDisplay::new("stuck", Behaviour::Block(gate.clone())),
        FakeDisplay::new("buggy", Behaviour::Crash),
    ]);
    let client = spawn(&source, budgets(Duration::from_millis(250)));

    let failures = [
        client.read_capabilities(&id("mute")).map(drop),
        client.read_vcp(&id("mute"), VcpCode::BRIGHTNESS).map(drop),
        client.write_vcp(&id("mute"), VcpCode::BRIGHTNESS, 10),
    ];
    let stuck = client.read_vcp(&id("stuck"), VcpCode::BRIGHTNESS);
    gate.open();
    let crashed = client.read_vcp(&id("buggy"), VcpCode::BRIGHTNESS);
    let after_crash = client.enumerate().map(drop);

    for failure in &failures {
        assert_transport(failure, NAK);
    }
    assert_eq!(stuck, Err(DdcError::Timeout));
    assert_transport(&crashed, "worker");
    assert_transport(&after_crash, "worker");
}

#[test]
fn unknown_monitor_is_looked_up_once_more_then_not_found() {
    let source = FakeDisplays::with([FakeDisplay::new("a", Behaviour::Answer)]);
    let client = spawn(&source, budgets(Duration::from_secs(1)));
    client.enumerate().unwrap();

    let result = client.read_vcp(&id("ghost"), VcpCode::BRIGHTNESS);

    assert_eq!(result, Err(DdcError::MonitorNotFound(id("ghost"))));
    assert_eq!(source.enumerations(), 2);
    assert!(source.calls().is_empty());
}

#[test]
fn first_request_enumerates_on_demand() {
    let source = FakeDisplays::with([FakeDisplay::new("a", Behaviour::Answer)]);
    let client = spawn(&source, budgets(Duration::from_secs(1)));

    let value = client.read_vcp(&id("a"), VcpCode::CONTRAST).unwrap();
    client.write_vcp(&id("a"), VcpCode::CONTRAST, 7).unwrap();

    assert_eq!(value, FAKE_VALUE);
    assert_eq!(source.enumerations(), 1);
    assert_eq!(
        source.calls(),
        [
            Call::Read("a", VcpCode::CONTRAST),
            Call::Write("a", VcpCode::CONTRAST, 7)
        ]
    );
}

#[test]
fn unknown_id_is_reported_at_once_without_enumerating() {
    let source = FakeDisplays::with([FakeDisplay::new("a", Behaviour::Answer)]);
    let clock = VirtualClock::new();
    let mut worker = Worker::new(source.clone(), RetryPolicy::default(), clock.clone());

    let result = worker.read_vcp(
        &id("a"),
        VcpCode::BRIGHTNESS,
        clock.now() + Duration::from_secs(1),
    );

    assert_eq!(result, Err(TransactError::UnknownMonitor));
    assert_eq!(source.enumerations(), 0);
    assert!(source.calls().is_empty());
}

/// The enumeration outlasts the VCP budget but not its own: the first read
/// on a fresh backend still answers (D-2026-09-26-ddc-backends-1).
#[test]
fn first_request_enumerates_under_the_enumeration_budget() {
    let source = FakeDisplays::with([FakeDisplay::new("a", Behaviour::Answer)]);
    let displays = SlowDisplays {
        displays: source.clone(),
        clock: SystemClock,
        cost: Duration::from_millis(250),
    };
    let budgets = DdcHiBudgets {
        vcp: Duration::from_millis(100),
        capabilities: Duration::from_millis(100),
        enumerate: Duration::from_secs(5),
    };
    let client = WorkerClient::spawn(displays, budgets, no_backoff()).unwrap();

    let read = client.read_vcp(&id("a"), VcpCode::BRIGHTNESS);
    let missing = client.write_vcp(&id("ghost"), VcpCode::BRIGHTNESS, 10);

    assert_eq!(read, Ok(FAKE_VALUE));
    assert_eq!(missing, Err(DdcError::MonitorNotFound(id("ghost"))));
    assert_eq!(source.enumerations(), 2);
    assert_eq!(source.calls(), [Call::Read("a", VcpCode::BRIGHTNESS)]);
}

#[test]
fn enumeration_replaces_the_display_table() {
    let source = FakeDisplays::with([FakeDisplay::new("old", Behaviour::Answer)]);
    let client = spawn(&source, budgets(Duration::from_secs(1)));
    client.enumerate().unwrap();
    source.plug([FakeDisplay::new("new", Behaviour::Answer)]);

    let listed = client.enumerate().unwrap();
    let old = client.read_vcp(&id("old"), VcpCode::BRIGHTNESS);
    let new = client.read_vcp(&id("new"), VcpCode::BRIGHTNESS);

    assert_eq!(
        listed
            .iter()
            .map(|info| info.id.clone())
            .collect::<Vec<_>>(),
        [id("new")]
    );
    assert_eq!(old, Err(DdcError::MonitorNotFound(id("old"))));
    assert_eq!(new, Ok(FAKE_VALUE));
}

#[test]
fn capabilities_reply_reaches_the_caller_as_text() {
    let source = FakeDisplays::with([FakeDisplay::new("a", Behaviour::Answer)]);
    let client = spawn(&source, budgets(Duration::from_secs(1)));

    let caps = client.read_capabilities(&id("a")).unwrap();

    assert_eq!(caps, "(prot(monitor)vcp(10 12))");
}

#[test]
fn retries_transient_errors_up_to_three_times_within_timeout_budget() {
    let backoff = Duration::from_millis(50);

    let recovers = read_on_worker(Behaviour::Flaky(2), Duration::from_secs(1));
    let persists = read_on_worker(Behaviour::Fail(FLAKY), Duration::from_secs(1));
    let short = read_on_worker(Behaviour::Fail(FLAKY), Duration::from_millis(80));

    assert_eq!(recovers.result, Ok(FAKE_VALUE));
    assert_eq!(
        (recovers.reads, recovers.sleeps),
        (3, vec![backoff, backoff])
    );
    assert_transport(&persists.result, "gave up after attempt 3 of 3");
    assert_eq!(
        (persists.reads, persists.sleeps),
        (3, vec![backoff, backoff])
    );
    assert_transport(&short.result, FLAKY);
    assert_eq!((short.reads, short.sleeps), (2, vec![backoff]));
}

#[test]
fn expired_write_is_never_sent_to_the_monitor() {
    let gate = Gate::default();
    let source = FakeDisplays::with([FakeDisplay::new("a", Behaviour::Block(gate.clone()))]);
    let client = spawn(&source, budgets(Duration::from_millis(20)));

    let stuck = client.read_vcp(&id("a"), VcpCode::BRIGHTNESS);
    let write = client.write_vcp(&id("a"), VcpCode::BRIGHTNESS, 80);
    gate.open();
    client.enumerate().unwrap();

    assert_eq!(stuck, Err(DdcError::Timeout));
    assert_eq!(write, Err(DdcError::Timeout));
    assert!(
        !source
            .calls()
            .iter()
            .any(|call| matches!(call, Call::Write(..))),
        "{:?}",
        source.calls()
    );
}

#[test]
fn monitor_gone_after_a_failure_is_not_found() {
    let run = read_on_worker(Behaviour::Vanish, Duration::from_secs(1));

    assert_eq!(run.result, Err(DdcError::MonitorNotFound(id("a"))));
    assert_eq!(run.reads, 3);
}

#[test]
fn deadline_spent_finding_the_monitor_sends_nothing() {
    let run = read_on_worker(Behaviour::Answer, Duration::ZERO);

    assert_eq!(run.result, Err(DdcError::Timeout));
    assert_eq!(run.reads, 0);
}

/// After the retries, a presence check that would end at or past the
/// deadline is skipped: the failure is a transport one, reported at once.
#[test]
fn failed_vcp_answers_transport_at_once_when_a_presence_check_cannot_fit() {
    let vcp = on_slow_worker(
        Behaviour::Fail(FLAKY),
        Duration::from_secs(1),
        read_brightness,
    );
    let tight = on_slow_worker(
        Behaviour::Fail(FLAKY),
        Duration::from_millis(100) + ENUMERATION,
        read_brightness,
    );

    assert_transport(&vcp.result, "gave up after attempt 3 of 3");
    assert_eq!(
        (vcp.enumerations, vcp.took),
        (1, Duration::from_millis(100))
    );
    assert_transport(&tight.result, FLAKY);
    assert_eq!(tight.enumerations, 1);
}

/// With the capabilities budget the presence check fits, so a vanished
/// monitor is told apart from a present one that keeps failing.
#[test]
fn failed_capabilities_read_checks_presence_when_it_fits() {
    let budget = Duration::from_secs(8);

    let vanished = on_slow_worker(Behaviour::Vanish, budget, Worker::read_capabilities);
    let present = on_slow_worker(Behaviour::Fail(FLAKY), budget, Worker::read_capabilities);

    assert_eq!(vanished.result, Err(DdcError::MonitorNotFound(id("a"))));
    assert_eq!(vanished.enumerations, 2);
    assert_transport(&present.result, "gave up after attempt 3 of 3");
    assert_eq!(
        (present.enumerations, present.took),
        (2, Duration::from_millis(100) + ENUMERATION)
    );
}
