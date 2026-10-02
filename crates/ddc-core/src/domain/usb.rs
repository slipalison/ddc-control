use std::fmt;
use std::str::FromStr;

/// A USB device as the switch follow knows it: its vendor and product ids,
/// plus its serial number when the device exposes one. The port it is
/// plugged into is not part of it: a switch may enumerate the device on
/// another port each time.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UsbDeviceId {
    /// USB vendor id (`idVendor`).
    pub vendor: u16,
    /// USB product id (`idProduct`).
    pub product: u16,
    /// Serial number, when the device has a non-empty one.
    pub serial: Option<String>,
}

impl UsbDeviceId {
    /// The id of a device; an empty serial counts as none.
    pub fn new(vendor: u16, product: u16, serial: Option<String>) -> Self {
        Self {
            vendor,
            product,
            serial: serial.filter(|serial| !serial.is_empty()),
        }
    }

    /// `vvvv:pppp`, the vendor and product ids without the serial.
    pub fn vendor_product(&self) -> String {
        format!("{:04x}:{:04x}", self.vendor, self.product)
    }
}

impl fmt::Display for UsbDeviceId {
    /// `vvvv:pppp[:serial]`, the ids in four lowercase hex digits.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.vendor_product())?;
        match &self.serial {
            Some(serial) => write!(f, ":{serial}"),
            None => Ok(()),
        }
    }
}

impl FromStr for UsbDeviceId {
    type Err = UsbDeviceIdError;

    /// Reads `vvvv:pppp[:serial]`: exactly four hex digits each for the
    /// vendor and the product, then the serial — everything after the second
    /// colon, colons included. An empty serial counts as none.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || UsbDeviceIdError(text.to_owned());
        let mut parts = text.splitn(3, ':');
        let vendor = parts.next().and_then(hex_id).ok_or_else(invalid)?;
        let product = parts.next().and_then(hex_id).ok_or_else(invalid)?;
        let serial = parts.next().map(str::to_owned);
        Ok(Self::new(vendor, product, serial))
    }
}

/// Four hex digits, as USB ids are written.
fn hex_id(digits: &str) -> Option<u16> {
    if digits.len() != 4 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    u16::from_str_radix(digits, 16).ok()
}

/// A text that is not a `vvvv:pppp[:serial]` USB device id.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is not a USB device id (vvvv:pppp[:serial])")]
pub struct UsbDeviceIdError(pub String);

/// Why the USB devices present could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("USB devices unreadable: {0}")]
pub struct UsbPresenceError(pub String);

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{UsbDeviceId, UsbDeviceIdError, UsbPresenceError};

    fn id(vendor: u16, product: u16, serial: Option<&str>) -> UsbDeviceId {
        UsbDeviceId::new(vendor, product, serial.map(str::to_owned))
    }

    #[test]
    fn displays_lowercase_four_digit_hex_and_the_serial() {
        assert_eq!(
            id(0x046d, 0xc52b, Some("AB12")).to_string(),
            "046d:c52b:AB12"
        );
        assert_eq!(id(0x1, 0xa, None).to_string(), "0001:000a");
        assert_eq!(
            id(0x046d, 0xc52b, Some("AB12")).vendor_product(),
            "046d:c52b"
        );
    }

    #[test]
    fn an_empty_serial_is_no_serial() {
        assert_eq!(id(0x046d, 0xc52b, Some("")), id(0x046d, 0xc52b, None));
        assert_eq!("046d:c52b:".parse(), Ok(id(0x046d, 0xc52b, None)));
    }

    #[test]
    fn parses_what_it_displays() {
        for device in [
            id(0x046d, 0xc52b, Some("AB12")),
            id(0x046d, 0xc077, None),
            id(0xffff, 0x0000, Some("with:colons")),
        ] {
            assert_eq!(device.to_string().parse(), Ok(device.clone()));
        }
        assert_eq!("046D:C52B".parse(), Ok(id(0x046d, 0xc52b, None)));
    }

    #[test]
    fn refuses_what_is_not_two_four_digit_hex_ids() {
        for text in [
            "",
            "046d",
            "046d:",
            "46d:c52b",
            "0046d:c52b",
            "046d:c5",
            "g46d:c52b",
            "+46d:c52b",
            " 046d:c52b",
        ] {
            assert_eq!(
                text.parse::<UsbDeviceId>(),
                Err(UsbDeviceIdError(text.to_owned())),
                "{text}"
            );
        }
    }

    #[test]
    fn ids_order_and_dedupe_in_a_set() {
        let set: BTreeSet<UsbDeviceId> = [
            id(0x046d, 0xc52b, Some("B")),
            id(0x046d, 0xc077, None),
            id(0x046d, 0xc52b, Some("B")),
        ]
        .into();

        assert_eq!(
            set.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["046d:c077", "046d:c52b:B"]
        );
    }

    #[test]
    fn errors_say_what_was_wrong() {
        assert_eq!(
            UsbDeviceIdError("x".to_owned()).to_string(),
            "\"x\" is not a USB device id (vvvv:pppp[:serial])"
        );
        assert_eq!(
            UsbPresenceError("permission denied".to_owned()).to_string(),
            "USB devices unreadable: permission denied"
        );
    }
}
