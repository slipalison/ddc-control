use std::cell::RefCell;

use assert_cmd::Command;
use clap::Parser;
use ddc_adapters::{BackendCall, FakeMonitor, InMemoryMonitorBackend};
use ddc_cli::fixture::{FIXTURE_CAPS, fixture_monitor};
use ddc_cli::{Cli, Exit, run};
use ddc_core::app::SoftwareOsd;
use ddc_core::domain::{MonitorId, MonitorInfo, VcpCode};

/// The built binary, serving the `--fake` monitor.
pub fn fake_cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ddc-cli"));
    command.arg("--fake");
    command
}

/// The built binary with no argument added.
pub fn bare_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ddc-cli"))
}

/// What one in-process run produced.
pub struct Outcome {
    pub exit: Exit,
    pub out: String,
    pub err: String,
    /// The backend's call log, including the enumeration.
    pub calls: Vec<BackendCall>,
    /// Monitors passed to the refresh callback, with how many backend
    /// calls had been made at that point.
    pub refreshed: Vec<(MonitorId, usize)>,
}

/// Runs `args` (without the program name) in-process against `backend`.
/// Arguments clap rejects end like the binary does: with [`Exit::Usage`]
/// and the backend untouched.
pub fn run_with(backend: &InMemoryMonitorBackend, args: &[&str]) -> Outcome {
    match Cli::try_parse_from(std::iter::once("ddc-cli").chain(args.iter().copied())) {
        Ok(cli) => run_parsed(backend, &cli),
        Err(usage) => Outcome {
            exit: Exit::Usage,
            out: String::new(),
            err: usage.to_string(),
            calls: Vec::new(),
            refreshed: Vec::new(),
        },
    }
}

fn run_parsed(backend: &InMemoryMonitorBackend, cli: &Cli) -> Outcome {
    let refreshed = RefCell::new(Vec::new());
    let refresh = |id: &MonitorId| {
        refreshed
            .borrow_mut()
            .push((id.clone(), backend.calls().len()));
    };
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let before = backend.calls().len();

    let exit = run(
        cli,
        &SoftwareOsd::new(backend.clone()),
        &refresh,
        &mut out,
        &mut err,
    );

    Outcome {
        exit,
        out: String::from_utf8_lossy(&out).into_owned(),
        err: String::from_utf8_lossy(&err).into_owned(),
        calls: backend.calls().split_off(before),
        refreshed: refreshed.into_inner(),
    }
}

/// A backend serving the `--fake` fixture monitor only.
pub fn fixture_backend() -> InMemoryMonitorBackend {
    backend_of([fixture_monitor()])
}

/// A backend serving `monitors` in order.
pub fn backend_of(monitors: impl IntoIterator<Item = FakeMonitor>) -> InMemoryMonitorBackend {
    monitors
        .into_iter()
        .fold(InMemoryMonitorBackend::builder(), |builder, monitor| {
            builder.monitor(monitor)
        })
        .build()
}

/// A monitor with the fixture's capabilities and a brightness of 50/100.
pub fn monitor(key: &str) -> FakeMonitor {
    FakeMonitor::new(MonitorInfo {
        id: MonitorId::new(key),
        manufacturer: None,
        model: None,
        serial: None,
    })
    .with_capabilities(FIXTURE_CAPS)
    .with_value(VcpCode::BRIGHTNESS, 50, 100)
}

pub fn id(key: &str) -> MonitorId {
    MonitorId::new(key)
}

/// Whether the log shows the monitor was read or written beyond the
/// enumeration.
pub fn touched_a_monitor(calls: &[BackendCall]) -> bool {
    calls
        .iter()
        .any(|call| !matches!(call, BackendCall::Enumerate))
}
