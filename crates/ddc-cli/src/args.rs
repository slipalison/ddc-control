//! Command-line arguments: the clap model and the parsers of `<VCP>` and
//! `<VALUE>`. Feature and value names come from the core's MCCS catalog;
//! none is listed here.

use clap::{Parser, Subcommand, ValueEnum};
use ddc_core::domain::VcpCode;
use ddc_core::domain::mccs_catalog::{catalog, catalog_entry, code_for_alias, value_for_name};

/// The catalog's name of the value every factory reset takes
/// (D-2026-09-26-full-osd-control-8).
const RESET_VALUE_NAME: &str = "Reset";

/// Control monitor settings over DDC/CI.
#[derive(Debug, Parser)]
#[command(name = "ddc-cli", version)]
pub struct Cli {
    /// Monitor to use: its id, its index in `list`, or a unique part of its
    /// id (case-insensitive). Needed when more than one monitor is attached.
    #[arg(short, long, global = true, value_name = "ID|INDEX")]
    pub monitor: Option<String>,

    /// Print the result as JSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Serve a fixed in-memory monitor instead of real hardware.
    #[arg(long, global = true, hide = true)]
    pub fake: bool,

    /// What to do.
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// List the monitors reachable over DDC/CI.
    List,
    /// Show a monitor's capabilities.
    Caps {
        /// Read the capabilities from the monitor again instead of the cache.
        #[arg(long)]
        refresh: bool,
    },
    /// Read a VCP feature.
    Get {
        /// VCP code: decimal, 0x-prefixed hex, or a feature name such as
        /// brightness, preset, input, volume or osd-language.
        #[arg(value_parser = parse_vcp)]
        vcp: VcpCode,
    },
    /// Write a VCP feature and read it back.
    Set {
        /// VCP code: decimal, 0x-prefixed hex, or a feature name such as
        /// brightness, preset, input, volume or osd-language.
        #[arg(value_parser = parse_vcp)]
        vcp: VcpCode,
        /// Value to write: decimal, 0x-prefixed hex, or a value name such as
        /// srgb, 6500k or hdmi-1 (case and punctuation are ignored).
        #[arg(value_parser = parse_value)]
        value: ValueArg,
        /// Confirm a write the core classifies as dangerous.
        #[arg(short, long)]
        yes: bool,
    },
    /// Restore factory defaults: every setting, or one group of them.
    Reset {
        /// What to restore.
        target: ResetTarget,
        /// Confirm the reset; every reset is dangerous.
        #[arg(short, long)]
        yes: bool,
    },
}

/// `<VALUE>` of `set`: a number, or the name of a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueArg {
    /// A raw value, written as is.
    Number(u16),
    /// The name of a value, as the catalog lists it for some feature.
    Name(String),
}

impl ValueArg {
    /// The raw value to write to `code`: the number as is, or the byte the
    /// catalog gives this name for `code`. A name `code` has no value for is
    /// a usage error, whose message is returned. Whether this monitor
    /// accepts the value is the core's decision, not this one.
    pub fn resolve(&self, code: VcpCode) -> Result<u16, String> {
        match self {
            Self::Number(value) => Ok(*value),
            Self::Name(name) => value_for_name(code, name)
                .map(u16::from)
                .ok_or_else(|| unknown_value(code, name)),
        }
    }
}

fn unknown_value(code: VcpCode, name: &str) -> String {
    let label = code_label(code);
    let names: Vec<&str> = catalog_entry(code)
        .map(|entry| entry.values.iter().map(|(_, known)| *known).collect())
        .unwrap_or_default();
    if names.is_empty() {
        return format!("{label} has no value named '{name}'; give it a number");
    }
    format!(
        "{label} has no value named '{name}'; use a number or one of: {}",
        names.join(", ")
    )
}

/// What `reset` restores. Each target is one MCCS factory-reset code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ResetTarget {
    /// Every setting (0x04).
    Factory,
    /// Luminance and contrast (0x05).
    BrightnessContrast,
    /// Geometry (0x06).
    Geometry,
    /// Color (0x08).
    Color,
}

impl ResetTarget {
    /// The factory-reset code of this target.
    pub fn code(self) -> VcpCode {
        match self {
            Self::Factory => VcpCode::RESTORE_FACTORY_DEFAULTS,
            Self::BrightnessContrast => VcpCode::RESTORE_FACTORY_LUMINANCE_CONTRAST,
            Self::Geometry => VcpCode::RESTORE_FACTORY_GEOMETRY,
            Self::Color => VcpCode::RESTORE_FACTORY_COLOR,
        }
    }

    /// The value a reset writes: `reset <target>` is `set <code> reset`.
    pub fn value(self) -> ValueArg {
        ValueArg::Name(RESET_VALUE_NAME.to_owned())
    }
}

/// The catalog alias of `code`, if it has one.
pub fn alias_of(code: VcpCode) -> Option<&'static str> {
    catalog_entry(code).and_then(|entry| entry.alias)
}

/// `0xNN` followed by the alias, when the code has one.
pub fn code_label(code: VcpCode) -> String {
    match alias_of(code) {
        Some(alias) => format!("{code} {alias}"),
        None => code.to_string(),
    }
}

/// Parses `<VCP>`: a catalog alias, or a number from 0 to 255.
fn parse_vcp(text: &str) -> Result<VcpCode, String> {
    code_for_alias(text)
        .or_else(|| {
            parse_number(text)
                .and_then(|n| u8::try_from(n).ok())
                .map(VcpCode)
        })
        .ok_or_else(|| {
            let aliases: Vec<&str> = catalog().iter().filter_map(|entry| entry.alias).collect();
            format!(
                "expected a code from 0 to 255 (decimal or 0x hex) or one of: {}",
                aliases.join(", ")
            )
        })
}

/// Parses `<VALUE>`: a number from 0 to 65535, or a name the catalog gives
/// a value of any feature. Which feature it names a value of is checked
/// once the command runs.
fn parse_value(text: &str) -> Result<ValueArg, String> {
    const EXPECTED: &str = "expected a value from 0 to 65535 (decimal or 0x hex) \
                            or a value name such as srgb or hdmi-1";
    if let Some(number) = parse_number(text) {
        return u16::try_from(number)
            .map(ValueArg::Number)
            .map_err(|_| EXPECTED.to_owned());
    }
    let named = catalog()
        .iter()
        .any(|entry| value_for_name(entry.code, text).is_some());
    if named {
        Ok(ValueArg::Name(text.to_owned()))
    } else {
        Err(EXPECTED.to_owned())
    }
}

/// A non-negative decimal number, or a hex one after `0x`/`0X`. Signs,
/// spaces and empty digit runs are rejected.
fn parse_number(text: &str) -> Option<u32> {
    let (digits, radix) = match text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        Some(hex) => (hex, 16),
        None => (text, 10),
    };
    let well_formed = !digits.is_empty() && digits.chars().all(|c| c.is_digit(radix));
    well_formed
        .then(|| u32::from_str_radix(digits, radix).ok())
        .flatten()
}

#[cfg(test)]
mod tests;
