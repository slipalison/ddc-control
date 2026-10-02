//! The USB switch follow: when every device learned from the switch has
//! left this machine, the monitor is to be switched to the input of the
//! other one (D-2026-10-02-usb-switch-follow-3).
//!
//! Pure and in logical time: one [`Follower::observe`] call is one read of
//! the devices present, whatever clock drives the reads.

use std::collections::BTreeSet;
use std::time::Duration;

use crate::domain::UsbDeviceId;

/// How often the devices present are read. A switch takes a second or so to
/// move them; polling twice a second needs no root and no new dependency.
pub const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Reads in a row a change must last to count (about 1.5 s): a hub that
/// re-enumerates hides its devices for a read or two without them leaving.
pub const DEBOUNCE_POLLS: u32 = 3;

/// How long learning watches for devices to leave after the click: time
/// enough to reach the switch and press its button.
pub const LEARN_WINDOW: Duration = Duration::from_secs(30);

/// Whether a leave of the learned devices may switch the monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Follow {
    /// The user turned the follow on: a leave fires.
    On,
    /// Off, or learning: a leave is tracked but fires nothing.
    Off,
}

/// The learned devices just left: switch the monitor, once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fire;

/// Where the follower stands, after the debounce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowerState {
    /// Waiting to see a learned device present: what a follower starts in,
    /// and where one with nothing learned stays.
    Waiting,
    /// At least one learned device is present: their leave will count.
    Armed,
    /// They all left, and that leave was consumed; one must come back first.
    Spent,
}

/// Watches the learned devices read after read and says when they have all
/// left: once per leave, never on arrival nor for a state found at start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Follower {
    learned: BTreeSet<UsbDeviceId>,
    state: FollowerState,
    absent_polls: u32,
}

impl Follower {
    /// A follower of `learned`, waiting to see one of them present.
    pub fn new(learned: BTreeSet<UsbDeviceId>) -> Self {
        Self {
            learned,
            state: FollowerState::Waiting,
            absent_polls: 0,
        }
    }

    /// The devices it follows.
    pub fn learned(&self) -> &BTreeSet<UsbDeviceId> {
        &self.learned
    }

    /// Where it stands now.
    pub fn state(&self) -> FollowerState {
        self.state
    }

    /// Takes one read of the devices `present`. Fires when every learned
    /// device has been absent for [`DEBOUNCE_POLLS`] reads in a row after at
    /// least one was seen, and `follow` is [`Follow::On`]. With
    /// [`Follow::Off`] that leave is consumed all the same.
    pub fn observe(&mut self, present: &BTreeSet<UsbDeviceId>, follow: Follow) -> Option<Fire> {
        if self.learned.iter().any(|device| present.contains(device)) {
            self.state = FollowerState::Armed;
            self.absent_polls = 0;
            return None;
        }
        if self.state != FollowerState::Armed {
            return None;
        }
        self.absent_polls += 1;
        if self.absent_polls < DEBOUNCE_POLLS {
            return None;
        }
        self.state = FollowerState::Spent;
        self.absent_polls = 0;
        (follow == Follow::On).then_some(Fire)
    }
}

/// What learning records: the devices present before that are no longer
/// present after.
pub fn learn(
    before: &BTreeSet<UsbDeviceId>,
    after: &BTreeSet<UsbDeviceId>,
) -> BTreeSet<UsbDeviceId> {
    before.difference(after).cloned().collect()
}

#[cfg(test)]
mod tests;
