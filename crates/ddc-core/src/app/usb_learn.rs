//! Learning the USB switch: which devices leave together when its button is
//! pressed (D-2026-10-02-usb-switch-follow-5).

use std::collections::BTreeSet;

use crate::app::usb_follow::{DEBOUNCE_POLLS, LEARN_WINDOW, POLL_INTERVAL, learn};
use crate::domain::UsbDeviceId;

/// Reads in [`LEARN_WINDOW`]: 60 at two reads a second.
const LEARN_POLLS: u128 = LEARN_WINDOW.as_millis() / POLL_INTERVAL.as_millis();

/// What a learning read concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearnStep {
    /// Still watching.
    Watching,
    /// These devices left together and stayed away.
    Learned(BTreeSet<UsbDeviceId>),
    /// Nothing left within the window.
    Expired,
}

/// One learning, from the click to its outcome: the devices present at the
/// click, then one read at a time until the same devices have stayed away
/// for [`DEBOUNCE_POLLS`] reads — those are learned — or [`LEARN_WINDOW`]
/// ran out. Pure, in logical time: one [`LearnSession::observe`] call is
/// one read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnSession {
    before: BTreeSet<UsbDeviceId>,
    gone: BTreeSet<UsbDeviceId>,
    steady_polls: u32,
    polls: u32,
}

impl LearnSession {
    /// Starts learning from the devices present at the click.
    pub fn start(before: BTreeSet<UsbDeviceId>) -> Self {
        Self {
            before,
            gone: BTreeSet::new(),
            steady_polls: 0,
            polls: 0,
        }
    }

    /// Takes one read of the devices `present`. Once it answers
    /// [`LearnStep::Learned`] or [`LearnStep::Expired`] the learning is over.
    pub fn observe(&mut self, present: &BTreeSet<UsbDeviceId>) -> LearnStep {
        self.polls += 1;
        let gone = learn(&self.before, present);
        if gone != self.gone {
            self.gone = gone;
            self.steady_polls = 0;
        }
        if !self.gone.is_empty() {
            self.steady_polls += 1;
        }
        if self.steady_polls >= DEBOUNCE_POLLS {
            return LearnStep::Learned(self.gone.clone());
        }
        if u128::from(self.polls) >= LEARN_POLLS {
            return LearnStep::Expired;
        }
        LearnStep::Watching
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::time::Duration;

    use super::{LEARN_POLLS, LearnSession, LearnStep};
    use crate::app::usb_follow::{DEBOUNCE_POLLS, LEARN_WINDOW, POLL_INTERVAL};
    use crate::domain::UsbDeviceId;

    fn keyboard() -> UsbDeviceId {
        UsbDeviceId::new(0x046d, 0xc31c, Some("KB0001".to_owned()))
    }

    fn mouse() -> UsbDeviceId {
        UsbDeviceId::new(0x046d, 0xc077, None)
    }

    fn webcam() -> UsbDeviceId {
        UsbDeviceId::new(0x046d, 0x0825, None)
    }

    fn set(devices: &[UsbDeviceId]) -> BTreeSet<UsbDeviceId> {
        devices.iter().cloned().collect()
    }

    fn session() -> LearnSession {
        LearnSession::start(set(&[keyboard(), mouse(), webcam()]))
    }

    /// Shows the session `present` for `polls` reads; the step of each.
    fn steps(session: &mut LearnSession, present: &[UsbDeviceId], polls: u32) -> Vec<LearnStep> {
        let present = set(present);
        (0..polls).map(|_| session.observe(&present)).collect()
    }

    fn watching(polls: u32) -> Vec<LearnStep> {
        (0..polls).map(|_| LearnStep::Watching).collect()
    }

    #[test]
    fn the_timings_are_those_of_the_decision() {
        assert_eq!(POLL_INTERVAL, Duration::from_millis(500));
        assert_eq!(DEBOUNCE_POLLS, 3);
        assert_eq!(LEARN_WINDOW, Duration::from_secs(30));
        assert_eq!(LEARN_POLLS, 60);
    }

    #[test]
    fn learns_the_devices_that_left_together_once_they_stayed_away() {
        let mut session = session();
        assert_eq!(
            steps(&mut session, &[keyboard(), mouse(), webcam()], 4),
            watching(4)
        );

        assert_eq!(
            steps(&mut session, &[webcam()], DEBOUNCE_POLLS - 1),
            watching(2)
        );
        assert_eq!(
            session.observe(&set(&[webcam()])),
            LearnStep::Learned(set(&[keyboard(), mouse()]))
        );
    }

    #[test]
    fn a_leave_shorter_than_the_debounce_learns_nothing() {
        let mut session = session();
        for _ in 0..5 {
            assert_eq!(
                steps(&mut session, &[webcam()], DEBOUNCE_POLLS - 1),
                watching(2)
            );
            assert_eq!(
                steps(&mut session, &[keyboard(), mouse(), webcam()], 1),
                watching(1)
            );
        }
    }

    #[test]
    fn devices_leaving_one_after_the_other_are_learned_together() {
        let mut session = session();
        assert_eq!(steps(&mut session, &[mouse(), webcam()], 2), watching(2));

        assert_eq!(
            steps(&mut session, &[webcam()], DEBOUNCE_POLLS - 1),
            watching(2)
        );
        assert_eq!(
            session.observe(&set(&[webcam()])),
            LearnStep::Learned(set(&[keyboard(), mouse()]))
        );
    }

    #[test]
    fn a_device_plugged_in_meanwhile_is_not_learned() {
        let stick = UsbDeviceId::new(0x0781, 0x5581, Some("STICK".to_owned()));
        let mut session = session();

        let outcome = steps(&mut session, &[webcam(), stick], DEBOUNCE_POLLS);

        assert_eq!(
            outcome.last(),
            Some(&LearnStep::Learned(set(&[keyboard(), mouse()])))
        );
    }

    #[test]
    fn expires_when_nothing_stayed_away_within_the_window() {
        let window = u32::try_from(LEARN_POLLS).unwrap();
        let mut session = session();
        let mut seen = steps(&mut session, &[keyboard(), mouse(), webcam()], window - 2);
        seen.extend(steps(&mut session, &[webcam()], 1));

        seen.extend(steps(&mut session, &[keyboard(), mouse(), webcam()], 1));

        assert_eq!(seen.len(), 60);
        assert_eq!(seen[..59], watching(59));
        assert_eq!(seen[59], LearnStep::Expired);
    }
}
