//! Running one parsed command against the core's driving port.

use std::io::Write;

use ddc_core::domain::{Confirm, DdcError, MonitorId};
use ddc_core::ports::MonitorControl;

use crate::args::{Cli, Command};
use crate::exit::{CliError, Exit};
use crate::output::{Format, Printer};
use crate::select::select_monitor;

/// Runs `cli` against `control`, printing the result to `out` and errors
/// to `err`, and returns the exit code.
///
/// Every command but `list` first enumerates the monitors and selects one.
/// `caps --refresh` calls `refresh` with the selected monitor before
/// reading its capabilities, so the caller can drop any cached copy.
/// `--yes` is the only source of [`Confirm::Yes`]; whether a write needs
/// it is decided by the core.
pub fn run(
    cli: &Cli,
    control: &impl MonitorControl,
    refresh: &dyn Fn(&MonitorId),
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Exit {
    let mut printer = Printer::new(format(cli), out, err);
    match execute(cli, control, refresh, &mut printer) {
        Ok(()) => Exit::Success,
        Err(error) => fail(&mut printer, &error),
    }
}

/// Reports an error raised before any command could run — typically a
/// backend that failed to start — and returns its exit code.
pub fn report_startup_failure(cli: &Cli, error: DdcError, err: &mut dyn Write) -> Exit {
    let mut out = std::io::sink();
    let mut printer = Printer::new(format(cli), &mut out, err);
    fail(
        &mut printer,
        &CliError::Ddc {
            monitor: None,
            error,
        },
    )
}

fn format(cli: &Cli) -> Format {
    if cli.json { Format::Json } else { Format::Text }
}

fn fail(printer: &mut Printer<'_>, error: &CliError) -> Exit {
    printer.error(error);
    error.exit()
}

fn execute(
    cli: &Cli,
    control: &impl MonitorControl,
    refresh: &dyn Fn(&MonitorId),
    printer: &mut Printer<'_>,
) -> Result<(), CliError> {
    let monitors = control.list_monitors().map_err(|error| CliError::Ddc {
        monitor: None,
        error,
    })?;
    let select = || select_monitor(&monitors, cli.monitor.as_deref());
    match &cli.command {
        Command::List => printer.list(&monitors),
        Command::Caps { refresh: again } => {
            let id = select()?;
            if *again {
                refresh(&id);
            }
            let caps = control.capabilities(&id).map_err(on(&id))?;
            printer.caps(&id, &caps);
        }
        Command::Get { vcp } => {
            let id = select()?;
            let reading = control.get_feature(&id, *vcp).map_err(on(&id))?;
            printer.get(&id, &reading);
        }
        Command::Set { vcp, value, yes } => {
            let id = select()?;
            let confirm = if *yes { Confirm::Yes } else { Confirm::No };
            let read_back = control
                .set_feature(&id, *vcp, *value, confirm)
                .map_err(on(&id))?;
            printer.set(&id, *vcp, *value, read_back);
        }
    }
    Ok(())
}

/// Wraps a core error with the monitor it happened on.
fn on(id: &MonitorId) -> impl Fn(DdcError) -> CliError + '_ {
    move |error| CliError::Ddc {
        monitor: Some(id.clone()),
        error,
    }
}
