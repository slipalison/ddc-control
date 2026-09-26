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
    Access, Capabilities, FeatureReading, MonitorId, MonitorInfo, VcpCode, VcpValue,
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
