//! Rendering of results as text or JSON.
//!
//! Results go to the output stream; errors and warnings go to the error
//! stream as text, in both formats, so a script can parse the output and
//! rely on the exit code. A failure to write either stream is ignored: there
//! is nowhere left to report it.

use std::fmt::Display;
use std::io::Write;

use ddc_core::domain::mccs_catalog::{Interpretation, catalog_entry, interpret};
use ddc_core::domain::{
    Access, Capabilities, DdcError, Feature, FeatureKind, FeatureReading, MonitorId, MonitorInfo,
    Risk, VcpCode, VcpValue,
};
use serde::Serialize;

use crate::args::{alias_of, code_label};

/// How results are printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Human-readable lines.
    Text,
    /// One JSON document per command.
    Json,
}

/// One line of `features`: a feature, whether the capabilities declare it,
/// and what reading it gave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureRow {
    /// The feature, as the capabilities and the catalog describe it.
    pub feature: Feature,
    /// Whether the capabilities declare the code; `false` for a probed one.
    pub declared: bool,
    /// The value read, or why none was. Never
    /// [`DdcError::MonitorNotFound`]: that ends the whole command.
    pub outcome: Result<VcpValue, DdcError>,
}

/// Whether reading a feature gave a value, and if not, why
/// (D-2026-09-26-full-osd-control-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeStatus {
    /// The monitor answered.
    Ok,
    /// The monitor said it does not support the code.
    Unsupported,
    /// The monitor did not answer, or not in time.
    Unresponsive,
}

impl ProbeStatus {
    fn of(outcome: &Result<VcpValue, DdcError>) -> Self {
        match outcome {
            Ok(_) => Self::Ok,
            Err(DdcError::UnsupportedFeature(_)) => Self::Unsupported,
            Err(_) => Self::Unresponsive,
        }
    }

    fn json(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Unsupported => "unsupported",
            Self::Unresponsive => "unresponsive",
        }
    }
}

/// Writes each command's result in the chosen [`Format`].
pub struct Printer<'w> {
    format: Format,
    out: &'w mut dyn Write,
    err: &'w mut dyn Write,
}

impl<'w> Printer<'w> {
    /// A printer writing results to `out` and diagnostics to `err`.
    pub fn new(format: Format, out: &'w mut dyn Write, err: &'w mut dyn Write) -> Self {
        Self { format, out, err }
    }

    /// Prints the monitors in enumeration order, numbered from 1.
    pub fn list(&mut self, monitors: &[MonitorInfo]) {
        if monitors.is_empty() {
            self.warn("no monitor found");
        }
        match self.format {
            Format::Text => {
                let lines = monitors
                    .iter()
                    .enumerate()
                    .map(|(i, m)| monitor_line(i + 1, m));
                self.emit_lines(lines);
            }
            Format::Json => {
                let dtos: Vec<MonitorDto<'_>> = monitors
                    .iter()
                    .enumerate()
                    .map(|(i, m)| MonitorDto::new(i + 1, m))
                    .collect();
                self.emit_json(&dtos);
            }
        }
    }

    /// Prints a monitor's parsed capabilities.
    pub fn caps(&mut self, monitor: &MonitorId, caps: &Capabilities) {
        match self.format {
            Format::Text => self.emit_lines(caps_lines(monitor, caps)),
            Format::Json => self.emit_json(&CapsDto::new(monitor, caps)),
        }
    }

    /// Prints a feature reading, warning when the capabilities do not
    /// declare the code — which is also how the core reports capabilities
    /// it could not read, so the warning names both causes.
    pub fn get(&mut self, monitor: &MonitorId, reading: &FeatureReading) {
        let code = reading.feature.code;
        if !reading.declared_in_capabilities {
            self.warn(format_args!(
                "{} is not declared in capabilities, or they could not be read; \
                 showing what the monitor answered",
                code_label(code)
            ));
        }
        match self.format {
            Format::Text => self.emit_lines([feature_line(code, reading.value)]),
            Format::Json => self.emit_json(&ReadingDto::new(monitor, reading)),
        }
    }

    /// Prints the value read back after writing `requested` to `code`,
    /// warning when the monitor did not apply it. A write-only feature is
    /// never read back, so only what was sent is printed.
    pub fn set(&mut self, monitor: &MonitorId, code: VcpCode, requested: u16, read_back: VcpValue) {
        if is_write_only(code) {
            self.sent(monitor, code, requested);
            return;
        }
        if read_back.current != requested {
            self.warn(format_args!(
                "the monitor accepted {} = {} but reads back {}; it may have ignored the write",
                code_label(code),
                number(requested),
                number(read_back.current)
            ));
        }
        match self.format {
            Format::Text => self.emit_lines([feature_line(code, read_back)]),
            Format::Json => {
                self.emit_json(&WriteDto::new(monitor, code, requested, read_back));
            }
        }
    }

    /// Prints the `features` table, one row per code in code order, first
    /// warning when the capabilities could not be read, so that no code
    /// counts as declared.
    pub fn features(&mut self, rows: &[FeatureRow], unreadable_caps: Option<&DdcError>) {
        if let Some(error) = unreadable_caps {
            self.warn(format_args!(
                "capabilities could not be read ({error}); no code counts as declared, \
                 and --probe reads every catalogued one"
            ));
        }
        match self.format {
            Format::Text => self.emit_lines(feature_table(rows)),
            Format::Json => {
                let dtos: Vec<FeatureDto> = rows.iter().map(FeatureDto::new).collect();
                self.emit_json(&dtos);
            }
        }
    }

    /// Prints a write that was sent and, by design, not read back: the
    /// feature is write-only (D-2026-09-26-full-osd-control-8).
    pub fn sent(&mut self, monitor: &MonitorId, code: VcpCode, value: u16) {
        match self.format {
            Format::Text => self.emit_lines([format!(
                "{}: sent {} (write-only, not read back)",
                code_label(code),
                value_text(code, value)
            )]),
            Format::Json => self.emit_json(&SentDto::new(monitor, code, value)),
        }
    }

    /// Reports an error on the error stream.
    pub fn error(&mut self, message: impl Display) {
        let _ = writeln!(self.err, "error: {message}");
    }

    fn warn(&mut self, message: impl Display) {
        let _ = writeln!(self.err, "warning: {message}");
    }

    fn emit_lines(&mut self, lines: impl IntoIterator<Item = String>) {
        for line in lines {
            let _ = writeln!(self.out, "{line}");
        }
    }

    fn emit_json(&mut self, value: &impl Serialize) {
        let _ = serde_json::to_writer_pretty(&mut *self.out, value);
        let _ = writeln!(self.out);
    }
}

/// One monitor as `list` prints it: 1-based index, id, then whatever EDID
/// details are known.
pub fn monitor_line(index: usize, monitor: &MonitorInfo) -> String {
    let details: Vec<&str> = [&monitor.manufacturer, &monitor.model, &monitor.serial]
        .into_iter()
        .filter_map(Option::as_deref)
        .collect();
    let line = format!("{index}  {}", monitor.id);
    if details.is_empty() {
        line
    } else {
        format!("{line}  ({})", details.join(" "))
    }
}

/// A value in decimal with its hex form, since some features are read as
/// quantities and others as codes.
fn number(value: u16) -> String {
    format!("{value} (0x{value:02X})")
}

/// [`number`], followed by what the catalog says `raw` means for `code`:
/// `1 (0x01) sRGB`, `14400 (0x3840) 144.00 Hz`.
fn value_text(code: VcpCode, raw: u16) -> String {
    match interpret(code, raw) {
        Some(meaning) => format!("{} {meaning}", number(raw)),
        None => number(raw),
    }
}

/// What the catalog says `raw` means for `code`, split as the JSON shows
/// it: a value name, or any other interpretation as text.
fn meaning(code: VcpCode, raw: u16) -> (Option<&'static str>, Option<String>) {
    match interpret(code, raw) {
        Some(Interpretation::Named(name)) => (Some(name), None),
        Some(other) => (None, Some(other.to_string())),
        None => (None, None),
    }
}

fn is_write_only(code: VcpCode) -> bool {
    catalog_entry(code).is_some_and(|entry| entry.access == Access::WriteOnly)
}

fn feature_line(code: VcpCode, value: VcpValue) -> String {
    format!(
        "{}: {}, max {}",
        code_label(code),
        value_text(code, value.current),
        number(value.max)
    )
}

fn kind_label(kind: FeatureKind) -> &'static str {
    match kind {
        FeatureKind::Continuous => "C",
        FeatureKind::NonContinuous => "NC",
        FeatureKind::Table => "T",
    }
}

fn access_label(access: Access) -> &'static str {
    match access {
        Access::ReadOnly => "RO",
        Access::WriteOnly => "WO",
        Access::ReadWrite => "RW",
    }
}

fn risk_label(risk: Risk) -> &'static str {
    match risk {
        Risk::Safe => "safe",
        Risk::Dangerous => "dangerous",
    }
}

fn source_label(declared: bool) -> &'static str {
    if declared { "caps" } else { "probe" }
}

/// The MCCS name of `code`, when the catalog knows it.
fn description(code: VcpCode) -> Option<&'static str> {
    catalog_entry(code).and_then(|entry| entry.name)
}

/// The value column of `features`: `current/max` and what it means, or why
/// there is no value.
fn outcome_text(code: VcpCode, outcome: &Result<VcpValue, DdcError>) -> String {
    match (outcome, ProbeStatus::of(outcome)) {
        (Ok(value), _) => match interpret(code, value.current) {
            Some(meaning) => format!("{}/{} {meaning}", value.current, value.max),
            None => format!("{}/{}", value.current, value.max),
        },
        (_, ProbeStatus::Unsupported) => "not supported by this monitor".to_owned(),
        (_, _) => "not responding".to_owned(),
    }
}

const FEATURE_HEADER: [&str; 8] = [
    "CODE",
    "NAME",
    "TYPE",
    "ACCESS",
    "RISK",
    "SOURCE",
    "VALUE",
    "DESCRIPTION",
];

fn feature_cells(row: &FeatureRow) -> [String; 8] {
    let code = row.feature.code;
    [
        code.to_string(),
        alias_of(code).unwrap_or("-").to_owned(),
        kind_label(row.feature.kind).to_owned(),
        access_label(row.feature.access).to_owned(),
        risk_label(row.feature.risk).to_owned(),
        source_label(row.declared).to_owned(),
        outcome_text(code, &row.outcome),
        description(code).unwrap_or("-").to_owned(),
    ]
}

/// A header and one line per row, columns aligned; nothing at all for no
/// row.
fn feature_table(rows: &[FeatureRow]) -> Vec<String> {
    if rows.is_empty() {
        return Vec::new();
    }
    let lines: Vec<[String; 8]> = std::iter::once(FEATURE_HEADER.map(str::to_owned))
        .chain(rows.iter().map(feature_cells))
        .collect();
    let mut widths = [0; 8];
    for line in &lines {
        for (width, cell) in widths.iter_mut().zip(line) {
            *width = (*width).max(cell.chars().count());
        }
    }
    lines
        .iter()
        .map(|line| {
            let padded: Vec<String> = line
                .iter()
                .zip(widths)
                .map(|(cell, width)| format!("{cell:<width$}"))
                .collect();
            padded.join("  ").trim_end().to_owned()
        })
        .collect()
}

fn hex_list(bytes: &[u8]) -> String {
    let hex: Vec<String> = bytes.iter().map(|byte| format!("0x{byte:02X}")).collect();
    hex.join(" ")
}

fn caps_lines(monitor: &MonitorId, caps: &Capabilities) -> Vec<String> {
    let fields = [
        ("protocol", caps.protocol.clone()),
        ("type", caps.monitor_type.clone()),
        ("model", caps.model.clone()),
        ("mccs version", mccs_version(caps)),
        (
            "commands",
            (!caps.commands.is_empty()).then(|| hex_list(&caps.commands)),
        ),
    ];
    let mut lines = vec![format!("monitor: {monitor}")];
    lines.extend(
        fields
            .into_iter()
            .filter_map(|(label, value)| value.map(|value| format!("{label}: {value}"))),
    );
    lines.push("vcp codes:".to_owned());
    lines.extend(caps.vcp.iter().map(|(code, values)| {
        let label = code_label(VcpCode(*code));
        match values {
            Some(values) => format!("  {label}: {}", hex_list(values)),
            None => format!("  {label}"),
        }
    }));
    lines
}

fn mccs_version(caps: &Capabilities) -> Option<String> {
    caps.mccs_version
        .map(|(major, minor)| format!("{major}.{minor}"))
}

#[derive(Serialize)]
struct MonitorDto<'a> {
    index: usize,
    id: &'a str,
    manufacturer: Option<&'a str>,
    model: Option<&'a str>,
    serial: Option<&'a str>,
}

impl<'a> MonitorDto<'a> {
    fn new(index: usize, monitor: &'a MonitorInfo) -> Self {
        Self {
            index,
            id: monitor.id.as_str(),
            manufacturer: monitor.manufacturer.as_deref(),
            model: monitor.model.as_deref(),
            serial: monitor.serial.as_deref(),
        }
    }
}

#[derive(Serialize)]
struct CapsDto<'a> {
    monitor: &'a str,
    protocol: Option<&'a str>,
    #[serde(rename = "type")]
    monitor_type: Option<&'a str>,
    model: Option<&'a str>,
    mccs_version: Option<String>,
    commands: &'a [u8],
    vcp: Vec<VcpEntryDto<'a>>,
}

impl<'a> CapsDto<'a> {
    fn new(monitor: &'a MonitorId, caps: &'a Capabilities) -> Self {
        Self {
            monitor: monitor.as_str(),
            protocol: caps.protocol.as_deref(),
            monitor_type: caps.monitor_type.as_deref(),
            model: caps.model.as_deref(),
            mccs_version: mccs_version(caps),
            commands: &caps.commands,
            vcp: caps
                .vcp
                .iter()
                .map(|(code, values)| VcpEntryDto {
                    code: *code,
                    values: values.as_deref(),
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct VcpEntryDto<'a> {
    code: u8,
    values: Option<&'a [u8]>,
}

/// `value_name` and `interpreted` appear only when the catalog gives the
/// value a meaning, so the JSON of a plain reading is unchanged.
#[derive(Serialize)]
struct ReadingDto<'a> {
    monitor: &'a str,
    code: u8,
    name: Option<&'static str>,
    current: u16,
    max: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_name: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interpreted: Option<String>,
    declared_in_capabilities: bool,
}

impl<'a> ReadingDto<'a> {
    fn new(monitor: &'a MonitorId, reading: &FeatureReading) -> Self {
        let code = reading.feature.code;
        let (value_name, interpreted) = meaning(code, reading.value.current);
        Self {
            monitor: monitor.as_str(),
            code: code.0,
            name: alias_of(code),
            current: reading.value.current,
            max: reading.value.max,
            value_name,
            interpreted,
            declared_in_capabilities: reading.declared_in_capabilities,
        }
    }
}

/// `value_name` and `interpreted` describe the value read back, as in
/// [`ReadingDto`].
#[derive(Serialize)]
struct WriteDto<'a> {
    monitor: &'a str,
    code: u8,
    name: Option<&'static str>,
    requested: u16,
    current: u16,
    max: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_name: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interpreted: Option<String>,
    applied: bool,
}

impl<'a> WriteDto<'a> {
    fn new(monitor: &'a MonitorId, code: VcpCode, requested: u16, read_back: VcpValue) -> Self {
        let (value_name, interpreted) = meaning(code, read_back.current);
        Self {
            monitor: monitor.as_str(),
            code: code.0,
            name: alias_of(code),
            requested,
            current: read_back.current,
            max: read_back.max,
            value_name,
            interpreted,
            applied: read_back.current == requested,
        }
    }
}

/// One `features` row. Every field is always present, `null` when there
/// is nothing to show (D-2026-09-26-full-osd-control-3).
#[derive(Serialize)]
struct FeatureDto {
    code: u8,
    name: Option<&'static str>,
    description: Option<&'static str>,
    kind: &'static str,
    access: &'static str,
    risk: &'static str,
    declared_in_capabilities: bool,
    probe_status: &'static str,
    current: Option<u16>,
    max: Option<u16>,
    value_name: Option<&'static str>,
    interpreted: Option<String>,
}

impl FeatureDto {
    fn new(row: &FeatureRow) -> Self {
        let code = row.feature.code;
        let value = row.outcome.as_ref().ok();
        let (value_name, interpreted) =
            value.map_or((None, None), |value| meaning(code, value.current));
        Self {
            code: code.0,
            name: alias_of(code),
            description: description(code),
            kind: kind_label(row.feature.kind),
            access: access_label(row.feature.access),
            risk: risk_label(row.feature.risk),
            declared_in_capabilities: row.declared,
            probe_status: ProbeStatus::of(&row.outcome).json(),
            current: value.map(|value| value.current),
            max: value.map(|value| value.max),
            value_name,
            interpreted,
        }
    }
}

/// A write to a write-only feature: what was sent, and that nothing was
/// read back.
#[derive(Serialize)]
struct SentDto<'a> {
    monitor: &'a str,
    code: u8,
    name: Option<&'static str>,
    requested: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_name: Option<&'static str>,
    read_back: bool,
}

impl<'a> SentDto<'a> {
    fn new(monitor: &'a MonitorId, code: VcpCode, requested: u16) -> Self {
        Self {
            monitor: monitor.as_str(),
            code: code.0,
            name: alias_of(code),
            requested,
            value_name: meaning(code, requested).0,
            read_back: false,
        }
    }
}

#[cfg(test)]
mod tests;
