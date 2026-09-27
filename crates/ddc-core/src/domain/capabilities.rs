use std::collections::BTreeMap;

use super::mccs_catalog::catalog_entry;
use super::{Access, DdcError, Feature, FeatureKind, VcpCode, risk_for_code};

/// A monitor's parsed MCCS capabilities string, e.g.
/// `(prot(monitor)type(LCD)model(RTK)cmds(01 02)vcp(10 12 14(01 02))mccs_ver(2.2))`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// `prot(...)` — protocol class, usually `monitor`.
    pub protocol: Option<String>,
    /// `type(...)` — display technology, e.g. `LCD`.
    pub monitor_type: Option<String>,
    /// `model(...)` — model name.
    pub model: Option<String>,
    /// `cmds(...)` — supported DDC/CI command opcodes.
    pub commands: Vec<u8>,
    /// `vcp(...)` — declared VCP codes; `Some(values)` when the code lists its
    /// discrete values in parentheses, `None` for a bare code.
    pub vcp: BTreeMap<u8, Option<Vec<u8>>>,
    /// `mccs_ver(major.minor)`.
    pub mccs_version: Option<(u8, u8)>,
    /// Any other top-level tag, name to raw inner text (nesting kept verbatim).
    pub unknown_tags: BTreeMap<String, String>,
}

/// A run of text followed by the inner text of the parenthesised group that
/// comes right after it (`None` for trailing text with no group).
type Segment<'a> = (&'a str, Option<&'a str>);

impl Capabilities {
    /// Parses a capabilities string as reported by a monitor.
    ///
    /// Tolerant by design: missing spaces between tags or around groups,
    /// concatenated or lowercase hex bytes, unknown tags and garbage after
    /// the closing parenthesis are all accepted, and tokens that are not hex
    /// bytes are skipped. Only parentheses that never close are an error.
    pub fn parse(raw: &str) -> Result<Self, DdcError> {
        let mut caps = Self::default();
        for (prefix, group) in segments(outer_body(raw)?)? {
            if let Some(value) = group {
                caps.apply_tag(tag_name(prefix), value)?;
            }
        }
        Ok(caps)
    }

    /// Whether the capabilities declare `code` in their `vcp(...)` list.
    pub fn declares(&self, code: VcpCode) -> bool {
        self.vcp.contains_key(&code.0)
    }

    /// The feature `code`: its kind and access come from the
    /// [MCCS catalog](super::mccs_catalog), whether these capabilities
    /// declare the code or not; its allowed values come only from these
    /// capabilities. A code outside the catalog is read-write, and
    /// non-continuous exactly when the capabilities list its values.
    pub fn feature(&self, code: VcpCode) -> Feature {
        let allowed_values = self.vcp.get(&code.0).cloned().flatten();
        let (kind, access) = match (catalog_entry(code), &allowed_values) {
            (Some(entry), _) => (entry.kind, entry.access),
            (None, Some(_)) => (FeatureKind::NonContinuous, Access::ReadWrite),
            (None, None) => (FeatureKind::Continuous, Access::ReadWrite),
        };
        Feature {
            code,
            kind,
            access,
            risk: risk_for_code(code),
            allowed_values,
        }
    }

    fn apply_tag(&mut self, name: &str, value: &str) -> Result<(), DdcError> {
        match name.to_ascii_lowercase().as_str() {
            "" => {}
            "prot" => self.protocol = Some(value.trim().to_owned()),
            "type" => self.monitor_type = Some(value.trim().to_owned()),
            "model" => self.model = Some(value.trim().to_owned()),
            "cmds" => self.commands.extend(hex_bytes(value)),
            "vcp" => self.add_vcp_entries(value)?,
            "mccs_ver" => self.mccs_version = parse_version(value),
            _ => {
                self.unknown_tags.insert(name.to_owned(), value.to_owned());
            }
        }
        Ok(())
    }

    fn add_vcp_entries(&mut self, content: &str) -> Result<(), DdcError> {
        for (prefix, group) in segments(content)? {
            let codes = hex_bytes(prefix);
            for code in &codes {
                self.vcp.insert(*code, None);
            }
            if let (Some(code), Some(values)) = (codes.last(), group) {
                self.vcp.insert(*code, Some(hex_bytes(values)));
            }
        }
        Ok(())
    }
}

/// Strips the outer parentheses and drops anything after the one that closes
/// them. Strings without outer parentheses are taken as the body itself.
fn outer_body(raw: &str) -> Result<&str, DdcError> {
    let trimmed = raw.trim_matches(|c: char| c.is_whitespace() || c.is_control());
    match trimmed.strip_prefix('(') {
        Some(inner) => Ok(&inner[..closing_paren(inner)?]),
        None => Ok(trimmed),
    }
}

/// Splits `text` into [`Segment`]s at its top-level parenthesised groups.
fn segments(text: &str) -> Result<Vec<Segment<'_>>, DdcError> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('(') {
        let inner = &rest[open + 1..];
        let close = closing_paren(inner)?;
        found.push((&rest[..open], Some(&inner[..close])));
        rest = &inner[close + 1..];
    }
    found.push((rest, None));
    Ok(found)
}

/// Byte index of the `)` closing a group whose `(` sits right before `text`.
fn closing_paren(text: &str) -> Result<usize, DdcError> {
    let mut depth = 0usize;
    for (index, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' if depth == 0 => return Ok(index),
            ')' => depth -= 1,
            _ => {}
        }
    }
    Err(DdcError::Transport(
        "unbalanced parentheses in capabilities string".to_owned(),
    ))
}

/// The identifier right before a group's `(`, ignoring junk in front of it.
fn tag_name(prefix: &str) -> &str {
    prefix
        .trim_end()
        .rsplit(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .next()
        .unwrap_or_default()
}

/// Hex bytes in the whitespace-separated tokens of `text`. A token of an even
/// number of hex digits is read as consecutive two-digit bytes (`0102` is
/// `01 02`); a single hex digit is one byte, for scalers that drop a leading
/// zero. Any other token — an odd run of three or more digits, or one with a
/// character that is not a hex digit — is ignored as a whole.
fn hex_bytes(text: &str) -> Vec<u8> {
    text.split_whitespace()
        .filter(|token| is_hex_byte_run(token))
        .flat_map(|token| token.as_bytes().chunks(2))
        .filter_map(|pair| std::str::from_utf8(pair).ok())
        .filter_map(|pair| u8::from_str_radix(pair, 16).ok())
        .collect()
}

/// Whether `token` is a single hex digit or an even-length run of them.
fn is_hex_byte_run(token: &str) -> bool {
    let whole_bytes = token.len() == 1 || token.len().is_multiple_of(2);
    whole_bytes && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn parse_version(text: &str) -> Option<(u8, u8)> {
    let (major, minor) = text.trim().split_once('.')?;
    Some((major.trim().parse().ok()?, minor.trim().parse().ok()?))
}

#[cfg(test)]
mod tests;
