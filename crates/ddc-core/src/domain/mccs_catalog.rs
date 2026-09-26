//! The MCCS 2.2 catalog: name, keyword, type, access, write risk and value
//! names of every VCP code observed on the dev monitor "RTK QHD HDR" — the
//! codes its capabilities declare plus the ones it answers without declaring
//! them (D-2026-09-26-full-osd-control-1, -7, -8).
//!
//! The table is closed on purpose: naming a code never seen on real hardware
//! could put a wrong label in front of a user. A code outside it keeps the
//! generic behaviour of [`Capabilities::feature`](super::Capabilities::feature)
//! and is [`Risk::Dangerous`]. Every lookup below reads this one table.

use std::fmt;

use super::{Access, FeatureKind, Risk, VcpCode};

use Access::{ReadOnly as RO, ReadWrite as RW, WriteOnly as WO};
use FeatureKind::{Continuous as C, NonContinuous as NC};
use Risk::{Dangerous, Safe};

/// What the core knows about one VCP code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogEntry {
    /// VCP code.
    pub code: VcpCode,
    /// MCCS name; `None` where the code's meaning is not confirmed.
    pub name: Option<&'static str>,
    /// The keyword a user types for the code: lowercase, kebab-case, unique.
    /// `None` for codes reachable only by number or by a dedicated command.
    pub alias: Option<&'static str>,
    /// How the value is interpreted.
    pub kind: FeatureKind,
    /// Allowed access directions.
    pub access: Access,
    /// Consequence class of writing it. Never `Dangerous` for a read-only
    /// code, so a refused write says "not writable", not "confirm it".
    pub risk: Risk,
    /// Names of the discrete values of a non-continuous code, by byte.
    pub values: &'static [(u8, &'static str)],
}

impl CatalogEntry {
    const fn with_values(self, values: &'static [(u8, &'static str)]) -> Self {
        Self { values, ..self }
    }
}

/// A catalog row without value names.
const fn row(
    code: u8,
    name: Option<&'static str>,
    alias: Option<&'static str>,
    kind: FeatureKind,
    access: Access,
    risk: Risk,
) -> CatalogEntry {
    CatalogEntry {
        code: VcpCode(code),
        name,
        alias,
        kind,
        access,
        risk,
        values: &[],
    }
}

/// The only value of a factory reset: MCCS ignores zero, and any non-zero
/// value triggers the reset (D-2026-09-26-full-osd-control-8).
const RESET: &[(u8, &str)] = &[(0x01, "Reset")];

const COLOR_PRESETS: &[(u8, &str)] = &[
    (0x01, "sRGB"),
    (0x02, "Display Native"),
    (0x04, "5000 K"),
    (0x05, "6500 K"),
    (0x06, "7500 K"),
    (0x08, "9300 K"),
    (0x0B, "User 1"),
];

const INPUT_SOURCES: &[(u8, &str)] = &[
    (0x01, "VGA-1"),
    (0x03, "DVI-1"),
    (0x04, "DVI-2"),
    (0x0F, "DisplayPort-1"),
    (0x10, "DisplayPort-2"),
    (0x11, "HDMI-1"),
    (0x12, "HDMI-2"),
];

const OSD_LANGUAGES: &[(u8, &str)] = &[
    (0x01, "Chinese (traditional)"),
    (0x02, "English"),
    (0x03, "French"),
    (0x04, "German"),
    (0x06, "Japanese"),
    (0x0A, "Spanish"),
    (0x0D, "Chinese (simplified)"),
];

const POWER_MODES: &[(u8, &str)] = &[
    (0x01, "On"),
    (0x04, "Off (DPM)"),
    (0x05, "Off (write-only)"),
];

/// Sorted by code, one row per code: code, MCCS name, alias, kind, access,
/// risk, value names.
#[rustfmt::skip]
const CATALOG: [CatalogEntry; 39] = [
    row(0x02, Some("New Control Value"),                           Some("new-control-value"),    NC, RW, Safe),
    row(0x04, Some("Restore Factory Defaults"),                    None,                         NC, WO, Dangerous).with_values(RESET),
    row(0x05, Some("Restore Factory Luminance/Contrast Defaults"), None,                         NC, WO, Dangerous).with_values(RESET),
    row(0x06, Some("Restore Factory Geometry Defaults"),           None,                         NC, WO, Dangerous).with_values(RESET),
    row(0x08, Some("Restore Factory Color Defaults"),              None,                         NC, WO, Dangerous).with_values(RESET),
    row(0x0B, Some("Color Temperature Increment"),                 Some("color-temp-increment"), C,  RO, Safe),
    row(0x0C, Some("Color Temperature Request"),                   Some("color-temp"),           C,  RW, Safe),
    row(0x10, Some("Luminance"),                                   Some("brightness"),           C,  RW, Safe),
    row(0x12, Some("Contrast"),                                    Some("contrast"),             C,  RW, Safe),
    row(0x14, Some("Select Color Preset"),                         Some("preset"),               NC, RW, Safe).with_values(COLOR_PRESETS),
    row(0x16, Some("Video Gain (Red)"),                            Some("red-gain"),             C,  RW, Safe),
    row(0x18, Some("Video Gain (Green)"),                          Some("green-gain"),           C,  RW, Safe),
    row(0x1A, Some("Video Gain (Blue)"),                           Some("blue-gain"),            C,  RW, Safe),
    row(0x1E, Some("Auto Setup"),                                  Some("auto-setup"),           NC, RW, Dangerous),
    row(0x20, Some("Horizontal Position"),                         Some("h-position"),           C,  RW, Dangerous),
    row(0x30, Some("Vertical Position"),                           Some("v-position"),           C,  RW, Dangerous),
    row(0x52, None,                                                None,                         C,  RO, Safe),
    row(0x60, Some("Input Source"),                                Some("input"),                NC, RW, Dangerous).with_values(INPUT_SOURCES),
    row(0x62, Some("Audio Speaker Volume"),                        Some("volume"),               C,  RW, Safe),
    row(0x6C, Some("Video Black Level (Red)"),                     Some("red-black-level"),      C,  RW, Safe),
    row(0x6E, Some("Video Black Level (Green)"),                   Some("green-black-level"),    C,  RW, Safe),
    row(0x70, Some("Video Black Level (Blue)"),                    Some("blue-black-level"),     C,  RW, Safe),
    row(0x7E, Some("Trapezoid"),                                   Some("trapezoid"),            C,  RW, Dangerous),
    row(0x87, Some("Sharpness"),                                   Some("sharpness"),            C,  RW, Safe),
    row(0xAC, Some("Horizontal Frequency"),                        Some("h-frequency"),          C,  RO, Safe),
    row(0xAE, Some("Vertical Frequency"),                          Some("v-frequency"),          C,  RO, Safe),
    row(0xB2, Some("Flat Panel Sub-Pixel Layout"),                 Some("subpixel-layout"),      NC, RO, Safe),
    row(0xB6, Some("Display Technology Type"),                     Some("display-technology"),   NC, RO, Safe),
    row(0xC6, None,                                                None,                         C,  RO, Safe),
    row(0xC8, Some("Display Controller Type"),                     Some("controller-type"),      NC, RO, Safe),
    row(0xC9, Some("Display Firmware Level"),                      Some("firmware-level"),       C,  RO, Safe),
    row(0xCA, Some("OSD/Button Control"),                          Some("osd-lock"),             NC, RW, Dangerous),
    row(0xCC, Some("OSD Language"),                                Some("osd-language"),         NC, RW, Safe).with_values(OSD_LANGUAGES),
    row(0xD6, Some("Power Mode"),                                  Some("power"),                NC, RW, Dangerous).with_values(POWER_MODES),
    row(0xDF, Some("VCP Version"),                                 Some("vcp-version"),          C,  RO, Safe),
    row(0xE6, Some("Manufacturer specific (0xE6)"),                None,                         C,  RW, Dangerous),
    row(0xF1, Some("Manufacturer specific (0xF1)"),                None,                         C,  RW, Dangerous),
    row(0xFD, None,                                                None,                         C,  RO, Safe),
    row(0xFF, None,                                                None,                         C,  RO, Safe),
];

/// Every code of the catalog, in order, read off the table itself.
const CODES: [VcpCode; CATALOG.len()] = {
    let mut codes = [VcpCode(0); CATALOG.len()];
    let mut index = 0;
    while index < CATALOG.len() {
        codes[index] = CATALOG[index].code;
        index += 1;
    }
    codes
};

/// Every catalog entry, sorted by code.
pub fn catalog() -> &'static [CatalogEntry] {
    &CATALOG
}

/// The catalog entry of `code`, if the code is catalogued.
pub fn catalog_entry(code: VcpCode) -> Option<&'static CatalogEntry> {
    let entries = catalog();
    entries
        .binary_search_by_key(&code, |entry| entry.code)
        .ok()
        .map(|index| &entries[index])
}

/// Every catalogued code, sorted.
pub fn catalog_codes() -> &'static [VcpCode] {
    &CODES
}

/// The name of value `byte` of `code`, when the catalog names it.
pub fn value_name(code: VcpCode, byte: u8) -> Option<&'static str> {
    catalog_entry(code)?
        .values
        .iter()
        .find(|(value, _)| *value == byte)
        .map(|(_, name)| *name)
}

/// The byte of the value of `code` called `name`. Case and every character
/// that is not a letter or a digit are ignored, so `6500k` names `6500 K`
/// and `displayport1` names `DisplayPort-1`.
pub fn value_for_name(code: VcpCode, name: &str) -> Option<u8> {
    catalog_entry(code)?
        .values
        .iter()
        .find(|(_, known)| same_value_name(known, name))
        .map(|(value, _)| *value)
}

/// Whether two value names are equal once normalized as
/// [`value_for_name`] describes.
fn same_value_name(left: &str, right: &str) -> bool {
    normalized(left).eq(normalized(right))
}

fn normalized(name: &str) -> impl Iterator<Item = char> + '_ {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
}

/// The code whose [`CatalogEntry::alias`] is `alias`, ignoring ASCII case.
pub fn code_for_alias(alias: &str) -> Option<VcpCode> {
    catalog()
        .iter()
        .find(|entry| {
            entry
                .alias
                .is_some_and(|known| known.eq_ignore_ascii_case(alias))
        })
        .map(|entry| entry.code)
}

/// A raw frequency in hundredths of a hertz, in hertz: `14400` is 144 Hz.
pub fn hertz(raw: u16) -> f64 {
    f64::from(raw) / 100.0
}

/// A raw version as `(major, minor)`: high byte, low byte. `0x0202` is 2.2.
pub fn version_from_u16(raw: u16) -> (u8, u8) {
    let [major, minor] = raw.to_be_bytes();
    (major, minor)
}

/// What a raw value of a catalogued code means.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interpretation {
    /// The name of a non-continuous value.
    Named(&'static str),
    /// A frequency, in hertz.
    Hertz(f64),
    /// A `major.minor` version.
    Version(u8, u8),
}

impl fmt::Display for Interpretation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => f.write_str(name),
            Self::Hertz(hertz) => write!(f, "{hertz:.2} Hz"),
            Self::Version(major, minor) => write!(f, "{major}.{minor}"),
        }
    }
}

/// What the current value `raw` of `code` means, when the catalog knows.
///
/// 0xAE counts hundredths of a hertz; 0xC9 and 0xDF are versions. A
/// non-continuous value is named after the low byte of `raw`, where MCCS
/// puts it. 0xAC stays raw: MCCS does not fix its unit the same way
/// (D-2026-09-26-full-osd-control-8).
pub fn interpret(code: VcpCode, raw: u16) -> Option<Interpretation> {
    match code {
        VcpCode::VERTICAL_FREQUENCY => Some(Interpretation::Hertz(hertz(raw))),
        VcpCode::FIRMWARE_LEVEL | VcpCode::VCP_VERSION => {
            let (major, minor) = version_from_u16(raw);
            Some(Interpretation::Version(major, minor))
        }
        _ => {
            let [_, low] = raw.to_be_bytes();
            value_name(code, low).map(Interpretation::Named)
        }
    }
}

#[cfg(test)]
mod tests;
