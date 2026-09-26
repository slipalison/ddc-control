//! Command-line arguments: the clap model and the parsers of `<VCP>` and
//! `<VALUE>`.

use clap::{Parser, Subcommand};
use ddc_core::domain::VcpCode;

/// Named shortcuts accepted for `<VCP>`, matched case-insensitively.
pub const SHORTCUTS: [(&str, VcpCode); 6] = [
    ("brightness", VcpCode::BRIGHTNESS),
    ("contrast", VcpCode::CONTRAST),
    ("input", VcpCode::INPUT_SOURCE),
    ("preset", VcpCode::COLOR_PRESET),
    ("volume", VcpCode::AUDIO_VOLUME),
    ("power", VcpCode::POWER_MODE),
];

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
        /// VCP code: decimal, 0x-prefixed hex, or one of brightness,
        /// contrast, input, preset, volume, power.
        #[arg(value_parser = parse_vcp)]
        vcp: VcpCode,
    },
    /// Write a VCP feature and read it back.
    Set {
        /// VCP code: decimal, 0x-prefixed hex, or one of brightness,
        /// contrast, input, preset, volume, power.
        #[arg(value_parser = parse_vcp)]
        vcp: VcpCode,
        /// Value to write: decimal or 0x-prefixed hex.
        #[arg(value_parser = parse_value)]
        value: u16,
        /// Confirm a write the core classifies as dangerous.
        #[arg(short, long)]
        yes: bool,
    },
}

/// The shortcut name of `code`, if it has one.
pub fn shortcut_name(code: VcpCode) -> Option<&'static str> {
    SHORTCUTS
        .iter()
        .find(|(_, shortcut)| *shortcut == code)
        .map(|(name, _)| *name)
}

/// Parses `<VCP>`: a shortcut name, or a number from 0 to 255.
fn parse_vcp(text: &str) -> Result<VcpCode, String> {
    let named = SHORTCUTS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(text))
        .map(|(_, code)| *code);
    named
        .or_else(|| {
            parse_number(text)
                .and_then(|n| u8::try_from(n).ok())
                .map(VcpCode)
        })
        .ok_or_else(|| {
            let names: Vec<&str> = SHORTCUTS.iter().map(|(name, _)| *name).collect();
            format!(
                "expected a code from 0 to 255 (decimal or 0x hex) or one of: {}",
                names.join(", ")
            )
        })
}

/// Parses `<VALUE>`: a number from 0 to 65535.
fn parse_value(text: &str) -> Result<u16, String> {
    parse_number(text)
        .and_then(|n| u16::try_from(n).ok())
        .ok_or_else(|| "expected a value from 0 to 65535 (decimal or 0x hex)".to_owned())
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
