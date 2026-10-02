use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use ddc_core::domain::UsbDeviceId;
use ddc_core::ports::UsbPresence;
use tempfile::TempDir;

use super::SysfsUsbPresence;

/// Writes a device directory as the kernel lays it out under
/// `/sys/bus/usb/devices`: one file per attribute, ending in a newline.
pub(super) fn plug(root: &Path, name: &str, attributes: &[(&str, &str)]) {
    let dir = root.join(name);
    fs::create_dir_all(&dir).unwrap();
    for (attribute, value) in attributes {
        fs::write(dir.join(attribute), format!("{value}\n")).unwrap();
    }
}

/// An interface of the keyboard: an entry with no `idVendor`, not a device.
/// Windows, whose runner also runs these tests, forbids the colon of the
/// name sysfs gives it.
const KEYBOARD_INTERFACE: &str = if cfg!(windows) { "1-2_1.0" } else { "1-2:1.0" };

fn keyboard(root: &Path) {
    plug(
        root,
        "1-2",
        &[
            ("idVendor", "046d"),
            ("idProduct", "c31c"),
            ("bDeviceClass", "00"),
            ("serial", "KB0001"),
        ],
    );
}

fn mouse(root: &Path) {
    plug(
        root,
        "1-3",
        &[
            ("idVendor", "046d"),
            ("idProduct", "c077"),
            ("bDeviceClass", "00"),
        ],
    );
}

/// The switch itself: a hub, like the root hub of each bus.
fn switch_hub(root: &Path) {
    plug(
        root,
        "1-1",
        &[
            ("idVendor", "05e3"),
            ("idProduct", "0610"),
            ("bDeviceClass", "09"),
            ("serial", "HUB0001"),
        ],
    );
    plug(
        root,
        "usb1",
        &[
            ("idVendor", "1d6b"),
            ("idProduct", "0002"),
            ("bDeviceClass", "09"),
        ],
    );
}

fn ids(texts: &[&str]) -> BTreeSet<UsbDeviceId> {
    texts.iter().map(|text| text.parse().unwrap()).collect()
}

#[test]
fn reads_vid_pid_and_serial_from_fake_root() {
    let root = TempDir::new().unwrap();
    keyboard(root.path());
    plug(
        root.path(),
        KEYBOARD_INTERFACE,
        &[("bInterfaceClass", "03")],
    );

    let present = SysfsUsbPresence::new(root.path()).present();

    assert_eq!(present, Ok(ids(&["046d:c31c:KB0001"])));
}

#[test]
fn device_without_serial_is_vid_pid_only() {
    let root = TempDir::new().unwrap();
    mouse(root.path());
    plug(
        root.path(),
        "1-4",
        &[
            ("idVendor", "0781"),
            ("idProduct", "5581"),
            ("serial", "  "),
        ],
    );

    let present = SysfsUsbPresence::new(root.path()).present();

    assert_eq!(present, Ok(ids(&["046d:c077", "0781:5581"])));
}

#[test]
fn skips_hubs() {
    let root = TempDir::new().unwrap();
    switch_hub(root.path());
    keyboard(root.path());
    mouse(root.path());

    let present = SysfsUsbPresence::new(root.path()).present();

    assert_eq!(present, Ok(ids(&["046d:c077", "046d:c31c:KB0001"])));
}

#[test]
fn missing_root_is_empty_set() {
    let parent = TempDir::new().unwrap();

    let present = SysfsUsbPresence::new(parent.path().join("absent")).present();

    assert_eq!(present, Ok(BTreeSet::new()));
}

#[test]
fn unplugged_device_disappears_between_two_reads() {
    let root = TempDir::new().unwrap();
    keyboard(root.path());
    mouse(root.path());
    let presence = SysfsUsbPresence::new(root.path());
    assert_eq!(
        presence.present(),
        Ok(ids(&["046d:c077", "046d:c31c:KB0001"]))
    );

    fs::remove_dir_all(root.path().join("1-2")).unwrap();

    assert_eq!(presence.present(), Ok(ids(&["046d:c077"])));
}
