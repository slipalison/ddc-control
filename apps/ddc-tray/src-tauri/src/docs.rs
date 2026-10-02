//! What the setup guide of the USB switch follow must say: the DDC/CI access
//! and the settings on BOTH machines, the input each one switches to, and
//! what turning the follow on consents to (D-2026-10-02-usb-switch-follow-8).
//! The guide is read from the repository when the tests are built.

/// The tests; the module is built for tests only.
mod tests {
    /// `docs/usb-switch-follow.md`, as committed.
    const GUIDE: &str = include_str!("../../../../docs/usb-switch-follow.md");

    /// The phrases of `wanted` the guide does not hold.
    fn missing<'a>(wanted: &[&'a str]) -> Vec<&'a str> {
        wanted
            .iter()
            .copied()
            .filter(|phrase| !GUIDE.contains(phrase))
            .collect()
    }

    #[test]
    fn usb_follow_doc_names_i2c_setup_for_both_machines() {
        let wanted = [
            "i2c-dev",
            "/dev/i2c-",
            "modules-load.d",
            "XDG_CONFIG_HOME",
            "usb-follow.json",
            "PC",
            "notebook",
            "0x0F",
            "0x10",
        ];

        assert_eq!(missing(&wanted), Vec::<&str>::new());
    }

    #[test]
    fn usb_follow_doc_states_the_consent_of_the_dangerous_write() {
        let wanted = [
            "Follow USB switch",
            "consent",
            "Dangerous",
            "0x60",
            "Learning never turns follow on and never writes to the monitor.",
        ];

        assert_eq!(missing(&wanted), Vec::<&str>::new());
    }
}
