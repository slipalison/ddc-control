use std::collections::BTreeSet;

use crate::domain::{UsbDeviceId, UsbPresenceError};

/// Driven port: the USB devices plugged into this machine right now.
///
/// Implementations leave hubs out — a USB switch is usually one, and it
/// says nothing about where the keyboard and the mouse are — and read
/// fresh on every call: the switch follow polls it.
pub trait UsbPresence: Send + Sync {
    /// The devices present now, hubs excluded.
    ///
    /// # Errors
    ///
    /// [`UsbPresenceError`] when the devices cannot be read at all.
    fn present(&self) -> Result<BTreeSet<UsbDeviceId>, UsbPresenceError>;
}
