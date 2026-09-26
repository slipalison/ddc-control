//! Running one parsed command against the core's driving port.

use std::io::Write;

use ddc_core::domain::{
    Capabilities, Confirm, DdcError, MonitorId, ProbedFeature, VcpCode, VcpValue,
};
use ddc_core::ports::MonitorControl;

use crate::args::{Cli, Command};
use crate::exit::{CliError, Exit};
use crate::output::{FeatureRow, Format, Printer};
use crate::select::select_monitor;

/// Runs `cli` against `control`, printing the result to `out` and errors
/// to `err`, and returns the exit code.
///
/// Every command but `list` first enumerates the monitors and selects one;
/// a value name that does not fit its feature is refused before that.
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
    let enumerate = || {
        control.list_monitors().map_err(|error| CliError::Ddc {
            monitor: None,
            error,
        })
    };
    let select = || Ok::<_, CliError>(select_monitor(&enumerate()?, cli.monitor.as_deref())?);
    match &cli.command {
        Command::List => printer.list(&enumerate()?),
        Command::Caps { refresh: again } => {
            let id = select()?;
            if *again {
                refresh(&id);
            }
            printer.caps(&id, &control.capabilities(&id).map_err(on(&id))?);
        }
        Command::Features { probe } => {
            let (rows, unreadable_caps) = feature_rows(control, &select()?, *probe)?;
            printer.features(&rows, unreadable_caps.as_ref());
        }
        Command::Get { vcp } => {
            let id = select()?;
            printer.get(&id, &control.get_feature(&id, *vcp).map_err(on(&id))?);
        }
        Command::Set { vcp, value, yes } => {
            let value = value.resolve(*vcp).map_err(CliError::Usage)?;
            let id = select()?;
            let read_back = write(control, &id, *vcp, value, *yes)?;
            printer.set(&id, *vcp, value, read_back);
        }
        Command::Reset { target, yes } => {
            let code = target.code();
            let value = target.value().resolve(code).map_err(CliError::Usage)?;
            let id = select()?;
            write(control, &id, code, value, *yes)?;
            printer.sent(&id, code, value);
        }
    }
    Ok(())
}

/// The rows of `features`: every code the capabilities declare, then with
/// `probe` every catalogued code they leave out, in code order — plus why
/// the capabilities could not be read, if they could not. Only reads. A
/// monitor found missing on any row ends the command
/// (D-2026-09-26-full-osd-control-3).
fn feature_rows(
    control: &impl MonitorControl,
    id: &MonitorId,
    probe: bool,
) -> Result<(Vec<FeatureRow>, Option<DdcError>), CliError> {
    let (caps, unreadable) = match control.capabilities(id) {
        Ok(caps) => (caps, None),
        Err(missing @ DdcError::MonitorNotFound(_)) => return Err(on(id)(missing)),
        Err(error) => (Capabilities::default(), Some(error)),
    };
    let mut rows = Vec::new();
    for code in caps.vcp.keys().map(|code| VcpCode(*code)) {
        let outcome = control.get_feature(id, code).map(|reading| reading.value);
        rows.push(feature_row(&caps, code, true, outcome).map_err(on(id))?);
    }
    let probed = if probe {
        control.probe_undeclared_features(id).map_err(on(id))?
    } else {
        Vec::new()
    };
    for ProbedFeature { code, outcome } in probed {
        let outcome = outcome.map(|reading| reading.value);
        rows.push(feature_row(&caps, code, false, outcome).map_err(on(id))?);
    }
    rows.sort_by_key(|row| row.feature.code);
    Ok((rows, unreadable))
}

/// The row of `code`, unless reading it found the monitor missing.
fn feature_row(
    caps: &Capabilities,
    code: VcpCode,
    declared: bool,
    outcome: Result<VcpValue, DdcError>,
) -> Result<FeatureRow, DdcError> {
    if let Err(missing @ DdcError::MonitorNotFound(_)) = outcome {
        return Err(missing);
    }
    Ok(FeatureRow {
        feature: caps.feature(code),
        declared,
        outcome,
    })
}

/// Writes through the core; `yes` is the user's `--yes`.
fn write(
    control: &impl MonitorControl,
    id: &MonitorId,
    code: VcpCode,
    value: u16,
    yes: bool,
) -> Result<VcpValue, CliError> {
    let confirm = if yes { Confirm::Yes } else { Confirm::No };
    control
        .set_feature(id, code, value, confirm)
        .map_err(on(id))
}

/// Wraps a core error with the monitor it happened on.
fn on(id: &MonitorId) -> impl Fn(DdcError) -> CliError + '_ {
    move |error| CliError::Ddc {
        monitor: Some(id.clone()),
        error,
    }
}
