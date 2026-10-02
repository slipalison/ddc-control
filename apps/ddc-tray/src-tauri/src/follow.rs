//! The USB switch follow's loop (D-2026-10-02-usb-switch-follow-2, -3, -7,
//! -8): on a thread of its own, it reads the USB devices present every
//! [`POLL_INTERVAL`], feeds them to the core's [`Follower`] and, when the
//! learned devices have all left and the user turned the follow on, writes
//! the recorded input to the recorded monitor — once, on another thread, so
//! the polling goes on while the monitor settles. It also runs the learning
//! the tray menu asks for.
//!
//! This is the one place the follow builds [`Confirm::Yes`]: the user gave
//! it by turning the follow on with complete settings.

use std::collections::BTreeSet;
use std::fmt::{self, Display};
use std::io;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use ddc_core::app::usb_follow::{Follow, Follower, FollowerState, LEARN_WINDOW, POLL_INTERVAL};
use ddc_core::app::usb_learn::{LearnSession, LearnStep};
use ddc_core::domain::{Confirm, MonitorId, UsbDeviceId, VcpCode};
use ddc_core::ports::UsbPresence;

use crate::commands::SharedOsd;
use crate::dto::UiError;
use crate::follow_config::{ConfigStore, FollowConfig, Incomplete, NotAFollowInput, Target};

/// Name of the thread that polls the USB devices.
pub const LOOP_THREAD: &str = "usb-follow";

/// Name of the thread each input switch runs on.
pub const SWITCH_THREAD: &str = "usb-follow-switch";

/// The line the app always prints on stderr when the follow switches a
/// monitor: `ddc-tray: follow: switching <monitor-id> to input 0x<hh>`.
pub fn switch_line(monitor: &MonitorId, input: u8) -> String {
    format!("ddc-tray: follow: switching {monitor} to input 0x{input:02x}")
}

/// Why a change of the follow's settings did not happen.
#[derive(Debug)]
pub enum FollowError {
    /// Turning the follow on needs what is listed.
    Incomplete(Incomplete),
    /// Not an input the follow switches to.
    NotAFollowInput(NotAFollowInput),
    /// The settings file could not be written: nothing changed.
    Save(io::Error),
}

impl Display for FollowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Incomplete(incomplete) => incomplete.fmt(f),
            Self::NotAFollowInput(code) => code.fmt(f),
            Self::Save(error) => write!(f, "the settings could not be saved: {error}"),
        }
    }
}

impl std::error::Error for FollowError {}

/// What the menu and the loop share: the settings in memory, the file they
/// are saved to, and a learning the menu asked for.
pub struct FollowState {
    store: ConfigStore,
    inner: Mutex<Inner>,
}

struct Inner {
    config: FollowConfig,
    learn_request: Option<MonitorId>,
}

/// The follow's state as the menu and the loop hold it.
pub type SharedFollow = Arc<FollowState>;

impl FollowState {
    /// The state over `config`, as loaded from `store`.
    pub fn new(store: ConfigStore, config: FollowConfig) -> Self {
        Self {
            store,
            inner: Mutex::new(Inner {
                config,
                learn_request: None,
            }),
        }
    }

    /// The settings now.
    pub fn config(&self) -> FollowConfig {
        self.lock().config.clone()
    }

    /// Turns the follow off, or on when the settings are complete, and
    /// answers the new settings.
    ///
    /// # Errors
    ///
    /// What is missing to turn it on, or why it could not be saved.
    pub fn toggle_enabled(&self) -> Result<FollowConfig, FollowError> {
        self.change(|config| config.enabled_toggled().map_err(FollowError::Incomplete))
    }

    /// Makes `code` the input to switch to, and answers the new settings.
    ///
    /// # Errors
    ///
    /// A code off the follow's inputs, or why it could not be saved.
    pub fn choose_input(&self, code: u8) -> Result<FollowConfig, FollowError> {
        self.change(|config| {
            config
                .with_target_input(code)
                .map_err(FollowError::NotAFollowInput)
        })
    }

    /// Asks the loop to learn the switch, with `monitor` as the one to
    /// switch; the loop starts at its next read.
    pub fn request_learning(&self, monitor: MonitorId) {
        self.lock().learn_request = Some(monitor);
    }

    fn take_learn_request(&self) -> Option<MonitorId> {
        self.lock().learn_request.take()
    }

    fn record_learned(
        &self,
        devices: BTreeSet<UsbDeviceId>,
        monitor: MonitorId,
    ) -> Result<FollowConfig, FollowError> {
        self.change(|config| Ok(config.with_learned(devices, monitor)))
    }

    /// Saves what `next` makes of the settings, and only then keeps it.
    fn change(
        &self,
        next: impl FnOnce(&FollowConfig) -> Result<FollowConfig, FollowError>,
    ) -> Result<FollowConfig, FollowError> {
        let mut inner = self.lock();
        let config = next(&inner.config)?;
        self.store.save(&config).map_err(FollowError::Save)?;
        inner.config = config.clone();
        Ok(config)
    }

    /// The settings are plain data, valid after any panic: a poisoned lock
    /// is recovered.
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Where the loop says what happens: in the app, stderr.
pub trait FollowOutput: Send + Sync {
    /// A line printed whatever the diagnostics switch: the switch line.
    fn announce(&self, line: &str);
    /// A diagnostic event, printed with `DDC_TRAY_DEBUG=1`.
    fn diagnose(&self, event: &str);
    /// Something that failed, which the user has nothing to act on.
    fn report(&self, action: &str, error: &dyn Display);
}

/// The app's stderr.
pub struct Stderr;

impl FollowOutput for Stderr {
    fn announce(&self, line: &str) {
        eprintln!("{line}");
    }

    fn diagnose(&self, event: &str) {
        crate::diagnose(event);
    }

    fn report(&self, action: &str, error: &dyn Display) {
        crate::report(action, error);
    }
}

/// A job the loop hands off: the write of an input.
pub type Job = Box<dyn FnOnce() + Send + 'static>;

/// Runs the loop's jobs somewhere else than the loop.
pub trait Executor: Send + Sync {
    /// Starts `job`.
    ///
    /// # Errors
    ///
    /// Why it could not be started.
    fn execute(&self, job: Job) -> io::Result<()>;
}

/// A thread of its own per job, named [`SWITCH_THREAD`].
pub struct OwnThread;

impl Executor for OwnThread {
    fn execute(&self, job: Job) -> io::Result<()> {
        thread::Builder::new()
            .name(SWITCH_THREAD.to_owned())
            .spawn(job)
            .map(drop)
    }
}

/// What the loop waits with between two reads.
pub trait Clock {
    /// Waits `duration`.
    fn sleep(&self, duration: Duration);
}

/// The wall clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn sleep(&self, duration: Duration) {
        thread::sleep(duration);
    }
}

/// A learning under way, and the monitor it records.
struct Learning {
    session: LearnSession,
    monitor: MonitorId,
}

/// The follow's loop over a [`UsbPresence`] and the core.
pub struct FollowLoop<P> {
    presence: P,
    follow: SharedFollow,
    osd: Result<SharedOsd, UiError>,
    output: Arc<dyn FollowOutput>,
    executor: Box<dyn Executor>,
    follower: Follower,
    learning: Option<Learning>,
    unreadable: bool,
}

impl<P: UsbPresence> FollowLoop<P> {
    /// The loop, with nothing read yet.
    pub fn new(
        presence: P,
        follow: SharedFollow,
        osd: Result<SharedOsd, UiError>,
        output: Arc<dyn FollowOutput>,
        executor: Box<dyn Executor>,
    ) -> Self {
        Self {
            presence,
            follow,
            osd,
            output,
            executor,
            follower: Follower::new(BTreeSet::new()),
            learning: None,
            unreadable: false,
        }
    }

    /// One read of the devices present, and what it leads to: a step of the
    /// learning under way, a step of the follower over the learned devices
    /// and, when they have all left with the follow on, the switch.
    pub fn tick(&mut self) {
        let Some(present) = self.read() else {
            return;
        };
        self.learn(&present);
        let config = self.follow.config();
        if self.follower.learned() != &config.devices {
            self.follower = Follower::new(config.devices.clone());
        }
        // Learning never writes (D-2026-10-02-usb-switch-follow-8).
        let target = config.active_target().filter(|_| self.learning.is_none());
        let follow = if target.is_some() {
            Follow::On
        } else {
            Follow::Off
        };
        let before = self.follower.state();
        let fired = self.follower.observe(&present, follow);
        self.say_edge(before, self.follower.state());
        if let (Some(_), Some(target)) = (fired, target) {
            self.switch(target);
        }
    }

    /// Reads, then waits [`POLL_INTERVAL`] on `clock`, for as long as
    /// `keep_going` says.
    pub fn run(&mut self, clock: &impl Clock, mut keep_going: impl FnMut() -> bool) {
        while keep_going() {
            self.tick();
            clock.sleep(POLL_INTERVAL);
        }
    }

    /// The devices present; a failure is reported once per run of failures
    /// and the read is skipped.
    fn read(&mut self) -> Option<BTreeSet<UsbDeviceId>> {
        match self.presence.present() {
            Ok(present) => {
                self.unreadable = false;
                Some(present)
            }
            Err(error) => {
                if !self.unreadable {
                    self.output.report("read the USB devices", &error);
                }
                self.unreadable = true;
                None
            }
        }
    }

    /// Starts the learning the menu asked for, from the devices `present`
    /// now, or takes a step of the one under way.
    fn learn(&mut self, present: &BTreeSet<UsbDeviceId>) {
        if let Some(monitor) = self.follow.take_learn_request() {
            self.learning = Some(Learning {
                session: LearnSession::start(present.clone()),
                monitor,
            });
            self.output.diagnose("follow: learning started");
            return;
        }
        let Some(learning) = &mut self.learning else {
            return;
        };
        match learning.session.observe(present) {
            LearnStep::Watching => {}
            LearnStep::Learned(devices) => {
                let monitor = learning.monitor.clone();
                self.learning = None;
                self.record(devices, monitor);
            }
            LearnStep::Expired => {
                self.learning = None;
                let why = format!("no USB device left within {} s", LEARN_WINDOW.as_secs());
                self.output.report("learn the USB switch", &why);
            }
        }
    }

    fn record(&self, devices: BTreeSet<UsbDeviceId>, monitor: MonitorId) {
        let count = devices.len();
        match self.follow.record_learned(devices, monitor) {
            Ok(_) => self
                .output
                .diagnose(&format!("follow: learned {count} USB devices")),
            Err(error) => self.output.report("record the learned USB switch", &error),
        }
    }

    fn say_edge(&self, before: FollowerState, after: FollowerState) {
        match (before, after) {
            (FollowerState::Waiting | FollowerState::Spent, FollowerState::Armed) => {
                self.output.diagnose("follow: learned devices present");
            }
            (FollowerState::Armed, FollowerState::Spent) => {
                self.output.diagnose("follow: learned devices absent");
            }
            _ => {}
        }
    }

    /// Says the switch line, then hands the write to the executor: one try,
    /// whose failure is only reported (D-2026-10-02-usb-switch-follow-7).
    fn switch(&self, target: Target) {
        self.output
            .announce(&switch_line(&target.monitor_id, target.input));
        let osd = self.osd.clone();
        let output = Arc::clone(&self.output);
        let job: Job = Box::new(move || write_input(osd, &target, output.as_ref()));
        if let Err(error) = self.executor.execute(job) {
            self.output.report("start the input switch", &error);
        }
    }
}

/// Writes the recorded input to the recorded monitor, confirmed: the user
/// consented by turning the follow on (D-2026-10-02-usb-switch-follow-8).
/// The core reads the input back; a monitor that switched away may not
/// answer that read, so a failure says the switch is unconfirmed.
fn write_input(osd: Result<SharedOsd, UiError>, target: &Target, output: &dyn FollowOutput) {
    let written = osd.map_err(|error| error.message).and_then(|osd| {
        osd.set_feature(
            &target.monitor_id,
            VcpCode::INPUT_SOURCE,
            u16::from(target.input),
            Confirm::Yes,
        )
        .map_err(|error| error.to_string())
    });
    if let Err(error) = written {
        let action = format!(
            "confirm the switch of {} to input 0x{:02x}",
            target.monitor_id, target.input
        );
        output.report(&action, &error);
    }
}

/// Starts the loop on a thread of its own, named [`LOOP_THREAD`], with the
/// app's stderr and a thread per input switch. It runs as long as the app.
///
/// # Errors
///
/// Why the thread could not be started.
pub fn spawn<P: UsbPresence + 'static>(
    presence: P,
    follow: SharedFollow,
    osd: Result<SharedOsd, UiError>,
) -> io::Result<JoinHandle<()>> {
    thread::Builder::new()
        .name(LOOP_THREAD.to_owned())
        .spawn(move || {
            FollowLoop::new(presence, follow, osd, Arc::new(Stderr), Box::new(OwnThread))
                .run(&SystemClock, || true);
        })
}

/// Fakes at every end of the loop: a sysfs tree in a temporary directory,
/// a stderr that records, a clock that never sleeps, jobs run in line.
#[cfg(test)]
pub(crate) mod fake {
    use std::cell::{Cell, RefCell};
    use std::collections::{BTreeSet, VecDeque};
    use std::fmt::Display;
    use std::fs;
    use std::io;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use ddc_core::domain::{UsbDeviceId, UsbPresenceError};
    use ddc_core::ports::UsbPresence;
    use tempfile::TempDir;

    use super::{Clock, Executor, FollowOutput, Job};

    /// A USB device as sysfs shows it.
    pub(crate) struct FakeDevice {
        pub(crate) entry: &'static str,
        vendor: &'static str,
        product: &'static str,
        class: &'static str,
        serial: Option<&'static str>,
    }

    pub(crate) const KEYBOARD: FakeDevice = FakeDevice {
        entry: "1-1.1",
        vendor: "046d",
        product: "c31c",
        class: "00",
        serial: Some("KB0001"),
    };

    pub(crate) const MOUSE: FakeDevice = FakeDevice {
        entry: "1-1.2",
        vendor: "046d",
        product: "c077",
        class: "00",
        serial: None,
    };

    /// The switch: a hub, which stays.
    pub(crate) const HUB: FakeDevice = FakeDevice {
        entry: "1-1",
        vendor: "05e3",
        product: "0610",
        class: "09",
        serial: None,
    };

    /// The ids of the keyboard and the mouse: what learning the switch gives.
    pub(crate) fn keyboard_and_mouse() -> BTreeSet<UsbDeviceId> {
        ["046d:c31c:KB0001", "046d:c077"]
            .iter()
            .map(|text| text.parse().unwrap())
            .collect()
    }

    /// A sysfs tree of USB devices in a temporary directory.
    pub(crate) struct FakeRoot {
        dir: TempDir,
    }

    impl FakeRoot {
        pub(crate) fn new() -> Self {
            Self {
                dir: TempDir::new().unwrap(),
            }
        }

        pub(crate) fn path(&self) -> &Path {
            self.dir.path()
        }

        /// Makes the tree hold exactly `devices`.
        pub(crate) fn hold(&self, devices: &[&FakeDevice]) {
            for entry in fs::read_dir(self.path()).unwrap() {
                fs::remove_dir_all(entry.unwrap().path()).unwrap();
            }
            for device in devices {
                let dir = self.path().join(device.entry);
                fs::create_dir(&dir).unwrap();
                let serial = device.serial.map(|serial| ("serial", serial));
                let attributes = [
                    ("idVendor", device.vendor),
                    ("idProduct", device.product),
                    ("bDeviceClass", device.class),
                ];
                for (name, value) in attributes.into_iter().chain(serial) {
                    fs::write(dir.join(name), format!("{value}\n")).unwrap();
                }
            }
        }
    }

    /// The reads of a scenario: `count` reads of each set of devices, in
    /// order.
    pub(crate) fn reads<'a>(parts: &[(&[&'a FakeDevice], usize)]) -> Vec<Vec<&'a FakeDevice>> {
        parts
            .iter()
            .flat_map(|(devices, count)| std::iter::repeat_n(devices.to_vec(), *count))
            .collect()
    }

    /// A clock that never sleeps: it counts the waits and, before each read,
    /// has the tree hold that read's devices.
    pub(crate) struct ScriptedClock<'a> {
        root: &'a FakeRoot,
        reads: &'a [Vec<&'a FakeDevice>],
        waits: RefCell<Vec<Duration>>,
    }

    impl<'a> ScriptedClock<'a> {
        pub(crate) fn new(root: &'a FakeRoot, reads: &'a [Vec<&'a FakeDevice>]) -> Self {
            if let Some(first) = reads.first() {
                root.hold(first);
            }
            Self {
                root,
                reads,
                waits: RefCell::new(Vec::new()),
            }
        }

        /// One `true` per read, then `false`.
        pub(crate) fn keep_going(&self) -> impl FnMut() -> bool + use<'a> {
            let left = Cell::new(self.reads.len());
            move || {
                let more = left.get() > 0;
                left.set(left.get().saturating_sub(1));
                more
            }
        }

        pub(crate) fn waits(&self) -> Vec<Duration> {
            self.waits.borrow().clone()
        }
    }

    impl Clock for ScriptedClock<'_> {
        fn sleep(&self, duration: Duration) {
            let mut waits = self.waits.borrow_mut();
            waits.push(duration);
            if let Some(next) = self.reads.get(waits.len()) {
                self.root.hold(next);
            }
        }
    }

    /// Whether the app prints a line whatever `DDC_TRAY_DEBUG` says.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Printed {
        Always,
        WithDebug,
    }

    /// Records the app's stderr, line by line, telling the lines it always
    /// prints from the diagnostics.
    #[derive(Default)]
    pub(crate) struct Recorder {
        lines: Mutex<Vec<(Printed, String)>>,
    }

    impl Recorder {
        pub(crate) fn new() -> Arc<Self> {
            Arc::new(Self::default())
        }

        /// The lines as the app prints them with `DDC_TRAY_DEBUG=1`.
        pub(crate) fn lines(&self) -> Vec<String> {
            self.printed(|_| true)
        }

        /// The lines the app prints without `DDC_TRAY_DEBUG`.
        pub(crate) fn lines_without_debug(&self) -> Vec<String> {
            self.printed(|printed| printed == Printed::Always)
        }

        fn printed(&self, shown: impl Fn(Printed) -> bool) -> Vec<String> {
            self.lines
                .lock()
                .unwrap()
                .iter()
                .filter(|(printed, _)| shown(*printed))
                .map(|(_, line)| line.clone())
                .collect()
        }

        /// The lines that start with `prefix`.
        pub(crate) fn lines_starting(&self, prefix: &str) -> Vec<String> {
            self.lines()
                .into_iter()
                .filter(|line| line.starts_with(prefix))
                .collect()
        }

        fn push(&self, printed: Printed, line: String) {
            self.lines.lock().unwrap().push((printed, line));
        }
    }

    impl FollowOutput for Recorder {
        fn announce(&self, line: &str) {
            self.push(Printed::Always, line.to_owned());
        }

        fn diagnose(&self, event: &str) {
            self.push(Printed::WithDebug, format!("ddc-tray: {event}"));
        }

        fn report(&self, action: &str, error: &dyn Display) {
            self.push(
                Printed::Always,
                format!("ddc-tray: could not {action}: {error}"),
            );
        }
    }

    /// A [`UsbPresence`] that answers what it was given, in order, then no
    /// device at all.
    pub(crate) struct ScriptedPresence {
        answers: Mutex<VecDeque<Result<BTreeSet<UsbDeviceId>, UsbPresenceError>>>,
    }

    impl ScriptedPresence {
        pub(crate) fn new(
            answers: impl IntoIterator<Item = Result<BTreeSet<UsbDeviceId>, UsbPresenceError>>,
        ) -> Self {
            Self {
                answers: Mutex::new(answers.into_iter().collect()),
            }
        }
    }

    impl UsbPresence for ScriptedPresence {
        fn present(&self) -> Result<BTreeSet<UsbDeviceId>, UsbPresenceError> {
            self.answers
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| Ok(BTreeSet::new()))
        }
    }

    /// Runs each job right away, on the loop's own thread.
    pub(crate) struct Inline;

    impl Executor for Inline {
        fn execute(&self, job: Job) -> io::Result<()> {
            job();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ddc_adapters::{BackendCall, FakeMonitor, InMemoryMonitorBackend, SysfsUsbPresence};
    use ddc_core::app::usb_follow::DEBOUNCE_POLLS;
    use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode};
    use tempfile::TempDir;

    use super::fake::{
        FakeDevice, FakeRoot, HUB, Inline, KEYBOARD, MOUSE, Recorder, ScriptedClock,
        keyboard_and_mouse, reads,
    };
    use super::{FollowLoop, FollowState, POLL_INTERVAL, SharedFollow};
    use crate::commands::SharedOsd;
    use crate::fixture::{RTK_ID, rtk_id, rtk_monitor};
    use crate::follow_config::{CONFIG_FILE, ConfigStore, FollowConfig};
    use crate::panel::tests::osd_with;

    pub(super) const DEBOUNCE: usize = DEBOUNCE_POLLS as usize;
    pub(super) const PLUGGED: &[&FakeDevice] = &[&HUB, &KEYBOARD, &MOUSE];
    pub(super) const GONE: &[&FakeDevice] = &[&HUB];
    pub(super) const SWITCH_TO_DP2: &str =
        "ddc-tray: follow: switching RTK-RTK-QHD-HDR-01010101 to input 0x10";

    /// A second monitor answering the input source, to switch instead.
    const DELL_ID: &str = "DEL-U2720Q-7";

    /// The follow over a fake tree, a fake monitor and settings in a
    /// temporary directory, with a stderr that records.
    pub(super) struct Rig {
        root: FakeRoot,
        _home: TempDir,
        pub(super) store: ConfigStore,
        pub(super) follow: SharedFollow,
        pub(super) backend: InMemoryMonitorBackend,
        pub(super) output: Arc<Recorder>,
        follow_loop: FollowLoop<SysfsUsbPresence>,
    }

    impl Rig {
        pub(super) fn new(
            config: FollowConfig,
            monitors: impl IntoIterator<Item = FakeMonitor>,
        ) -> Self {
            let home = TempDir::new().unwrap();
            let store = ConfigStore::new(home.path().join("ddc-control").join(CONFIG_FILE));
            store.save(&config).unwrap();
            let follow: SharedFollow = Arc::new(FollowState::new(store.clone(), config));
            let (osd, backend) = osd_with(monitors);
            let osd: SharedOsd = Arc::new(osd);
            let root = FakeRoot::new();
            let output = Recorder::new();
            let follow_loop = FollowLoop::new(
                SysfsUsbPresence::new(root.path()),
                follow.clone(),
                Ok(osd),
                output.clone(),
                Box::new(Inline),
            );
            Self {
                root,
                _home: home,
                store,
                follow,
                backend,
                output,
                follow_loop,
            }
        }

        /// Runs the loop through `parts` with the fake clock; each read waits
        /// one poll interval.
        pub(super) fn run(&mut self, parts: &[(&[&FakeDevice], usize)]) {
            let reads = reads(parts);
            let clock = ScriptedClock::new(&self.root, &reads);
            self.follow_loop.run(&clock, clock.keep_going());
            assert_eq!(clock.waits(), vec![POLL_INTERVAL; reads.len()]);
        }

        pub(super) fn writes(&self) -> Vec<BackendCall> {
            self.backend
                .calls()
                .into_iter()
                .filter(|call| matches!(call, BackendCall::WriteVcp(..)))
                .collect()
        }

        pub(super) fn switch_lines(&self) -> Vec<String> {
            self.output.lines_starting("ddc-tray: follow: switching")
        }
    }

    pub(super) fn follow_to(monitor: MonitorId, input: u8, enabled: bool) -> FollowConfig {
        FollowConfig {
            enabled,
            devices: keyboard_and_mouse(),
            monitor_id: Some(monitor),
            target_input: Some(input),
        }
    }

    fn dell_monitor() -> FakeMonitor {
        FakeMonitor::new(MonitorInfo {
            id: MonitorId::new(DELL_ID),
            manufacturer: Some("DEL".to_owned()),
            model: Some("U2720Q".to_owned()),
            serial: Some("7".to_owned()),
        })
        .with_capabilities("(prot(monitor)vcp(10 60(0F 10 11 12)))")
        .with_value(VcpCode::BRIGHTNESS, 30, 100)
        .with_value(VcpCode::INPUT_SOURCE, 0x0F, 0x12)
    }

    #[test]
    fn leaving_learned_devices_writes_0x60_target_exactly_once_on_configured_monitor() {
        let mut rig = Rig::new(follow_to(rtk_id(), 0x10, true), [rtk_monitor()]);

        rig.run(&[(PLUGGED, 2), (GONE, DEBOUNCE + 5)]);

        assert_eq!(
            rig.writes(),
            [BackendCall::WriteVcp(rtk_id(), VcpCode::INPUT_SOURCE, 0x10)]
        );
        assert_eq!(
            rig.output.lines(),
            [
                "ddc-tray: follow: learned devices present",
                "ddc-tray: follow: learned devices absent",
                SWITCH_TO_DP2,
            ]
        );
        assert_eq!(rig.output.lines_without_debug(), [SWITCH_TO_DP2]);
    }

    #[test]
    fn arrival_writes_nothing() {
        let mut rig = Rig::new(follow_to(rtk_id(), 0x10, true), [rtk_monitor()]);

        rig.run(&[(GONE, 2 * DEBOUNCE), (PLUGGED, 2 * DEBOUNCE)]);

        assert_eq!(rig.writes(), []);
        assert_eq!(rig.switch_lines(), Vec::<String>::new());
        assert_eq!(
            rig.output.lines(),
            ["ddc-tray: follow: learned devices present"]
        );
        assert_eq!(rig.output.lines_without_debug(), Vec::<String>::new());
    }

    #[test]
    fn disabled_writes_nothing() {
        let mut rig = Rig::new(follow_to(rtk_id(), 0x10, false), [rtk_monitor()]);

        rig.run(&[
            (PLUGGED, 2),
            (GONE, 3 * DEBOUNCE),
            (PLUGGED, 1),
            (GONE, 3 * DEBOUNCE),
        ]);

        assert_eq!(rig.writes(), []);
        assert_eq!(rig.switch_lines(), Vec::<String>::new());
        assert_eq!(
            rig.output
                .lines_starting("ddc-tray: follow: learned devices")
                .len(),
            4
        );
        assert_eq!(rig.output.lines_without_debug(), Vec::<String>::new());
    }

    #[test]
    fn silent_ddc_write_is_reported_and_loop_keeps_polling() {
        let mute = rtk_monitor().with_vcp_failure(VcpCode::INPUT_SOURCE, DdcError::Timeout);
        let mut rig = Rig::new(follow_to(rtk_id(), 0x10, true), [mute]);
        let failed = format!(
            "ddc-tray: could not confirm the switch of {RTK_ID} to input 0x10: \
             monitor did not respond in time"
        );

        rig.run(&[
            (PLUGGED, 1),
            (GONE, DEBOUNCE + 3),
            (PLUGGED, 1),
            (GONE, DEBOUNCE + 2),
        ]);

        let attempt = BackendCall::WriteVcp(rtk_id(), VcpCode::INPUT_SOURCE, 0x10);
        assert_eq!(rig.writes(), [attempt.clone(), attempt]);
        assert_eq!(
            rig.output.lines(),
            [
                "ddc-tray: follow: learned devices present",
                "ddc-tray: follow: learned devices absent",
                SWITCH_TO_DP2,
                &failed,
                "ddc-tray: follow: learned devices present",
                "ddc-tray: follow: learned devices absent",
                SWITCH_TO_DP2,
                &failed,
            ]
        );
        assert_eq!(
            rig.output.lines_without_debug(),
            [SWITCH_TO_DP2, &failed, SWITCH_TO_DP2, &failed]
        );
    }

    #[test]
    fn writes_only_configured_monitor_and_code_0x60() {
        let dell = MonitorId::new(DELL_ID);
        let mut rig = Rig::new(
            follow_to(dell.clone(), 0x11, true),
            [rtk_monitor(), dell_monitor()],
        );

        rig.run(&[
            (PLUGGED, 1),
            (GONE, DEBOUNCE),
            (PLUGGED, 1),
            (GONE, DEBOUNCE),
        ]);

        let switch = BackendCall::WriteVcp(dell.clone(), VcpCode::INPUT_SOURCE, 0x11);
        assert_eq!(rig.writes(), [switch.clone(), switch]);
        let touched_rtk = rig.backend.calls().into_iter().any(|call| match call {
            BackendCall::Enumerate => false,
            BackendCall::ReadCapabilities(id)
            | BackendCall::ReadVcp(id, _)
            | BackendCall::WriteVcp(id, ..) => id == rtk_id(),
        });
        assert!(!touched_rtk, "{:?}", rig.backend.calls());
    }
}

/// Learning, failures and the runtime pieces, beyond the tests the phase's
/// definition of done names.
#[cfg(test)]
mod learning_tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::io;
    use std::sync::{Arc, mpsc};
    use std::time::{Duration, Instant};

    use ddc_adapters::BackendCall;
    use ddc_core::domain::{UsbPresenceError, VcpCode};
    use tempfile::TempDir;

    use super::fake::{Inline, Recorder, ScriptedPresence, keyboard_and_mouse};
    use super::tests::{DEBOUNCE, GONE, PLUGGED, Rig, SWITCH_TO_DP2, follow_to};
    use super::{
        Clock, Executor, FollowLoop, FollowState, Job, LOOP_THREAD, OwnThread, SWITCH_THREAD,
        SystemClock, spawn, switch_line,
    };
    use crate::dto::{ErrorKind, UiError};
    use crate::fixture::{rtk_id, rtk_monitor};
    use crate::follow_config::{CONFIG_FILE, ConfigStore, FollowConfig, LoadOutcome};
    use crate::panel::tests::osd_with;

    #[test]
    fn learning_records_the_devices_that_left_with_the_requested_monitor_and_writes_nothing() {
        let mut rig = Rig::new(FollowConfig::default(), [rtk_monitor()]);
        rig.follow.request_learning(rtk_id());

        rig.run(&[(PLUGGED, 2), (GONE, DEBOUNCE)]);

        let learned = FollowConfig::default().with_learned(keyboard_and_mouse(), rtk_id());
        assert_eq!(rig.follow.config(), learned);
        assert_eq!(rig.store.load(), LoadOutcome::Loaded(learned));
        assert_eq!(rig.writes(), []);
        assert_eq!(
            rig.output.lines(),
            [
                "ddc-tray: follow: learning started",
                "ddc-tray: follow: learned 2 USB devices"
            ]
        );

        rig.run(&[(PLUGGED, 1)]);
        assert_eq!(
            rig.output.lines().last().map(String::as_str),
            Some("ddc-tray: follow: learned devices present")
        );
    }

    #[test]
    fn learning_while_the_follow_is_on_writes_nothing_and_turns_it_off() {
        let mut rig = Rig::new(follow_to(rtk_id(), 0x10, true), [rtk_monitor()]);
        rig.run(&[(PLUGGED, 1)]);
        rig.follow.request_learning(rtk_id());

        rig.run(&[(PLUGGED, 1), (GONE, DEBOUNCE + 2)]);

        assert_eq!(rig.writes(), []);
        assert_eq!(rig.switch_lines(), Vec::<String>::new());
        assert_eq!(rig.follow.config(), follow_to(rtk_id(), 0x10, false));
        assert_eq!(
            rig.output.lines(),
            [
                "ddc-tray: follow: learned devices present",
                "ddc-tray: follow: learning started",
                "ddc-tray: follow: learned 2 USB devices",
                "ddc-tray: follow: learned devices absent",
            ]
        );
    }

    #[test]
    fn a_learning_where_nothing_left_expires_and_records_nothing() {
        let mut rig = Rig::new(FollowConfig::default(), [rtk_monitor()]);
        let before = fs::read(rig.store.path()).unwrap();
        rig.follow.request_learning(rtk_id());

        rig.run(&[(PLUGGED, 61)]);

        assert_eq!(
            rig.output.lines(),
            [
                "ddc-tray: follow: learning started",
                "ddc-tray: could not learn the USB switch: no USB device left within 30 s",
            ]
        );
        assert_eq!(fs::read(rig.store.path()).unwrap(), before);
        assert_eq!(rig.follow.config(), FollowConfig::default());
    }

    /// A loop over `presence` that follows the RTK to DisplayPort 2.
    fn loop_over(
        presence: ScriptedPresence,
        osd: Result<crate::commands::SharedOsd, UiError>,
        executor: Box<dyn Executor>,
    ) -> (FollowLoop<ScriptedPresence>, Arc<Recorder>, TempDir) {
        let home = TempDir::new().unwrap();
        let store = ConfigStore::new(home.path().join(CONFIG_FILE));
        let follow = Arc::new(FollowState::new(store, follow_to(rtk_id(), 0x10, true)));
        let output = Recorder::new();
        let follow_loop = FollowLoop::new(presence, follow, osd, output.clone(), executor);
        (follow_loop, output, home)
    }

    fn unreadable() -> Result<BTreeSet<ddc_core::domain::UsbDeviceId>, UsbPresenceError> {
        Err(UsbPresenceError("Input/output error".to_owned()))
    }

    #[test]
    fn a_failed_read_is_reported_once_per_run_and_counts_as_no_read() {
        let (osd, backend) = osd_with([rtk_monitor()]);
        let answers = [Ok(keyboard_and_mouse()), unreadable(), unreadable()]
            .into_iter()
            .chain([Ok(BTreeSet::new()), Ok(BTreeSet::new())])
            .chain([
                unreadable(),
                unreadable(),
                unreadable(),
                Ok(BTreeSet::new()),
            ]);
        let (mut follow_loop, output, _home) = loop_over(
            ScriptedPresence::new(answers),
            Ok(Arc::new(osd)),
            Box::new(Inline),
        );

        for _ in 0..9 {
            follow_loop.tick();
        }

        let unreadable =
            "ddc-tray: could not read the USB devices: USB devices unreadable: Input/output error";
        assert_eq!(
            output.lines(),
            [
                "ddc-tray: follow: learned devices present",
                unreadable,
                unreadable,
                "ddc-tray: follow: learned devices absent",
                SWITCH_TO_DP2,
            ]
        );
        let writes: Vec<_> = backend
            .calls()
            .into_iter()
            .filter(|call| matches!(call, BackendCall::WriteVcp(..)))
            .collect();
        assert_eq!(
            writes,
            [BackendCall::WriteVcp(rtk_id(), VcpCode::INPUT_SOURCE, 0x10)]
        );
    }

    #[test]
    fn a_switch_without_a_core_is_reported() {
        let presence = ScriptedPresence::new([Ok(keyboard_and_mouse())]);
        let missing = UiError {
            kind: ErrorKind::BackendUnavailable,
            message: "no DDC/CI backend".to_owned(),
        };
        let (mut follow_loop, output, _home) = loop_over(presence, Err(missing), Box::new(Inline));

        for _ in 0..=DEBOUNCE {
            follow_loop.tick();
        }

        assert_eq!(
            output.lines()[2..],
            [
                SWITCH_TO_DP2.to_owned(),
                "ddc-tray: could not confirm the switch of RTK-RTK-QHD-HDR-01010101 to input \
                 0x10: no DDC/CI backend"
                    .to_owned(),
            ]
        );
    }

    struct Refusing;

    impl Executor for Refusing {
        fn execute(&self, _job: Job) -> io::Result<()> {
            Err(io::Error::other("no thread left"))
        }
    }

    #[test]
    fn a_switch_that_cannot_start_is_reported() {
        let (osd, backend) = osd_with([rtk_monitor()]);
        let presence = ScriptedPresence::new([Ok(keyboard_and_mouse())]);
        let (mut follow_loop, output, _home) =
            loop_over(presence, Ok(Arc::new(osd)), Box::new(Refusing));

        for _ in 0..=DEBOUNCE {
            follow_loop.tick();
        }

        assert_eq!(
            output.lines()[2..],
            [
                SWITCH_TO_DP2,
                "ddc-tray: could not start the input switch: no thread left"
            ]
        );
        assert_eq!(backend.calls(), []);
    }

    #[test]
    fn a_change_that_cannot_be_saved_changes_nothing() {
        let home = TempDir::new().unwrap();
        fs::write(home.path().join("ddc-control"), "a file, not a directory").unwrap();
        let store = ConfigStore::new(home.path().join("ddc-control").join(CONFIG_FILE));
        let follow = FollowState::new(store, follow_to(rtk_id(), 0x10, false));

        let toggled = follow.toggle_enabled();
        let chosen = follow.choose_input(0x11);

        assert!(
            toggled
                .unwrap_err()
                .to_string()
                .starts_with("the settings could not be saved: "),
        );
        assert!(chosen.is_err());
        assert_eq!(follow.config(), follow_to(rtk_id(), 0x10, false));
        assert_eq!(
            follow.choose_input(0x13).unwrap_err().to_string(),
            "0x13 is not an input the follow switches to"
        );
        let nothing = FollowState::new(
            ConfigStore::new(home.path().join("x")),
            FollowConfig::default(),
        );
        assert_eq!(
            nothing.toggle_enabled().unwrap_err().to_string(),
            "learn the USB switch and pick the target input first \
             (no learned devices, no monitor, no target input)"
        );
    }

    #[test]
    fn the_switch_line_names_the_monitor_and_the_input_in_two_hex_digits() {
        assert_eq!(
            switch_line(&rtk_id(), 0x0F),
            "ddc-tray: follow: switching RTK-RTK-QHD-HDR-01010101 to input 0x0f"
        );
        assert_eq!(switch_line(&rtk_id(), 0x10), SWITCH_TO_DP2);
    }

    #[test]
    fn each_switch_runs_on_a_thread_of_its_own() {
        let (sender, receiver) = mpsc::channel();

        OwnThread
            .execute(Box::new(move || {
                let name = std::thread::current().name().map(str::to_owned);
                sender.send(name).unwrap();
            }))
            .unwrap();

        let name = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(name.as_deref(), Some(SWITCH_THREAD));
    }

    #[test]
    fn the_system_clock_waits_for_real() {
        let start = Instant::now();

        SystemClock.sleep(Duration::from_millis(5));

        assert!(start.elapsed() >= Duration::from_millis(5));
    }

    #[test]
    fn the_loop_runs_on_a_named_thread_of_its_own() {
        let home = TempDir::new().unwrap();
        let store = ConfigStore::new(home.path().join(CONFIG_FILE));
        let follow = Arc::new(FollowState::new(store, FollowConfig::default()));
        let (osd, _backend) = osd_with([rtk_monitor()]);

        let handle = spawn(ScriptedPresence::new([]), follow, Ok(Arc::new(osd))).unwrap();

        assert_eq!(handle.thread().name(), Some(LOOP_THREAD));
        assert!(!handle.is_finished());
    }
}
