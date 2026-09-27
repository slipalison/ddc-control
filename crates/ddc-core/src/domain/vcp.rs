use std::fmt;

/// A VCP (Virtual Control Panel) feature code as defined by VESA MCCS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VcpCode(pub u8);

impl VcpCode {
    /// 0x04 — restore all factory defaults.
    pub const RESTORE_FACTORY_DEFAULTS: Self = Self(0x04);
    /// 0x05 — restore factory luminance and contrast defaults.
    pub const RESTORE_FACTORY_LUMINANCE_CONTRAST: Self = Self(0x05);
    /// 0x06 — restore factory geometry defaults.
    pub const RESTORE_FACTORY_GEOMETRY: Self = Self(0x06);
    /// 0x08 — restore factory color defaults.
    pub const RESTORE_FACTORY_COLOR: Self = Self(0x08);
    /// 0x10 — luminance (brightness).
    pub const BRIGHTNESS: Self = Self(0x10);
    /// 0x12 — contrast.
    pub const CONTRAST: Self = Self(0x12);
    /// 0x14 — select color preset.
    pub const COLOR_PRESET: Self = Self(0x14);
    /// 0x16 — video gain, red.
    pub const RED_GAIN: Self = Self(0x16);
    /// 0x18 — video gain, green.
    pub const GREEN_GAIN: Self = Self(0x18);
    /// 0x1A — video gain, blue.
    pub const BLUE_GAIN: Self = Self(0x1A);
    /// 0x60 — input source.
    pub const INPUT_SOURCE: Self = Self(0x60);
    /// 0x62 — audio speaker volume.
    pub const AUDIO_VOLUME: Self = Self(0x62);
    /// 0x87 — sharpness.
    pub const SHARPNESS: Self = Self(0x87);
    /// 0xAE — vertical frequency, in hundredths of a hertz.
    pub const VERTICAL_FREQUENCY: Self = Self(0xAE);
    /// 0xC9 — display firmware level.
    pub const FIRMWARE_LEVEL: Self = Self(0xC9);
    /// 0xCA — OSD enable/lock.
    pub const OSD_LOCK: Self = Self(0xCA);
    /// 0xCC — OSD language.
    pub const OSD_LANGUAGE: Self = Self(0xCC);
    /// 0xD6 — power mode.
    pub const POWER_MODE: Self = Self(0xD6);
    /// 0xDF — VCP (MCCS) version.
    pub const VCP_VERSION: Self = Self(0xDF);
    /// 0xE0 — first code of the manufacturer-specific range (0xE0..=0xFF).
    pub const MANUFACTURER_SPECIFIC_START: Self = Self(0xE0);
}

impl fmt::Display for VcpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02X}", self.0)
    }
}

/// The value of a VCP feature as reported by the monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VcpValue {
    /// Current value.
    pub current: u16,
    /// Maximum value the monitor reports for this feature.
    pub max: u16,
}

#[cfg(test)]
mod tests {
    use super::VcpCode;

    #[test]
    fn displays_code_as_two_digit_uppercase_hex() {
        assert_eq!(VcpCode::BRIGHTNESS.to_string(), "0x10");
        assert_eq!(VcpCode(0x0b).to_string(), "0x0B");
        assert_eq!(VcpCode(0xff).to_string(), "0xFF");
    }
}
