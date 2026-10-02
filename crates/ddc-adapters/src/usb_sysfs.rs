//! The USB devices present, read from a sysfs tree such as
//! `/sys/bus/usb/devices` (D-2026-10-02-usb-switch-follow-2, -3). The tree
//! is handed in: only a composition root names the real one.

use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use ddc_core::domain::{UsbDeviceId, UsbPresenceError};
use ddc_core::ports::UsbPresence;

/// `bDeviceClass` of a hub, root hubs included.
const HUB_CLASS: &str = "09";

/// [`UsbPresence`] over a sysfs tree of USB devices: one directory per
/// device, with its `idVendor`, `idProduct` and optional `serial` files.
/// Interfaces (no `idVendor`), hubs and entries with a malformed id are
/// left out; an entry that vanishes while it is read is left out too.
#[derive(Debug, Clone)]
pub struct SysfsUsbPresence {
    root: PathBuf,
}

impl SysfsUsbPresence {
    /// Reads the devices under `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl UsbPresence for SysfsUsbPresence {
    /// A missing tree is no device at all: a machine without USB.
    fn present(&self) -> Result<BTreeSet<UsbDeviceId>, UsbPresenceError> {
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(BTreeSet::new()),
            Err(error) => {
                return Err(UsbPresenceError(format!(
                    "{}: {error}",
                    self.root.display()
                )));
            }
        };
        Ok(entries
            .filter_map(Result::ok)
            .filter_map(|entry| device_id(&entry.path()))
            .collect())
    }
}

/// The id of the device in `dir`, unless it is no device, a hub or
/// malformed.
fn device_id(dir: &Path) -> Option<UsbDeviceId> {
    let vendor = hex_attribute(dir, "idVendor")?;
    let product = hex_attribute(dir, "idProduct")?;
    if attribute(dir, "bDeviceClass").as_deref() == Some(HUB_CLASS) {
        return None;
    }
    Some(UsbDeviceId::new(vendor, product, attribute(dir, "serial")))
}

/// A sysfs attribute, trimmed; `None` when it cannot be read.
fn attribute(dir: &Path, name: &str) -> Option<String> {
    let bytes = fs::read(dir.join(name)).ok()?;
    Some(String::from_utf8_lossy(&bytes).trim().to_owned())
}

/// A USB id attribute: one to four hex digits.
fn hex_attribute(dir: &Path, name: &str) -> Option<u16> {
    let digits = attribute(dir, name)?;
    if digits.is_empty() || digits.len() > 4 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    u16::from_str_radix(&digits, 16).ok()
}

#[cfg(test)]
mod tests;

/// What the follow must survive in a real tree, outside the tests the
/// phase's definition of done names.
#[cfg(test)]
mod robustness_tests {
    use std::collections::BTreeSet;

    use ddc_core::ports::UsbPresence;
    use tempfile::TempDir;

    use super::SysfsUsbPresence;
    use super::tests::plug;

    // Windows may answer "not found" when a file is listed as a directory.
    #[cfg(unix)]
    #[test]
    fn a_root_that_cannot_be_listed_is_an_error() {
        use std::fs;

        use ddc_core::domain::UsbPresenceError;

        let parent = TempDir::new().unwrap();
        let file = parent.path().join("devices");
        fs::write(&file, "").unwrap();

        let present = SysfsUsbPresence::new(&file).present();

        let prefix = format!("{}: ", file.display());
        assert!(
            matches!(&present, Err(UsbPresenceError(reason)) if reason.starts_with(&prefix)),
            "{present:?}"
        );
    }

    #[test]
    fn entries_with_a_malformed_or_missing_id_are_left_out() {
        let root = TempDir::new().unwrap();
        plug(
            root.path(),
            "1-1",
            &[("idVendor", "zz6d"), ("idProduct", "c077")],
        );
        plug(
            root.path(),
            "1-2",
            &[("idVendor", "046d0"), ("idProduct", "c077")],
        );
        plug(
            root.path(),
            "1-3",
            &[("idVendor", ""), ("idProduct", "c077")],
        );
        plug(root.path(), "1-4", &[("idVendor", "046d")]);
        plug(
            root.path(),
            "1-5",
            &[("idVendor", "+46d"), ("idProduct", "c077")],
        );
        plug(
            root.path(),
            "1-6",
            &[("idVendor", "46d"), ("idProduct", "c077")],
        );

        let present = SysfsUsbPresence::new(root.path()).present();

        let expected: BTreeSet<_> = ["046d:c077".parse().unwrap()].into();
        assert_eq!(present, Ok(expected));
    }
}
