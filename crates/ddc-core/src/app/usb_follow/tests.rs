use std::collections::BTreeSet;
use std::time::Duration;

use super::{Fire, Follow, Follower, FollowerState, POLL_INTERVAL, learn};
use crate::domain::UsbDeviceId;

/// Reads in a row of absence after which the learned devices count as gone
/// (D-2026-10-02-usb-switch-follow-3): written out, so the tests pin it.
const LEAVE: u32 = 3;

fn keyboard() -> UsbDeviceId {
    UsbDeviceId::new(0x046d, 0xc31c, Some("KB0001".to_owned()))
}

fn mouse() -> UsbDeviceId {
    UsbDeviceId::new(0x046d, 0xc077, None)
}

/// A device that stays on this machine whatever the switch does.
fn webcam() -> UsbDeviceId {
    UsbDeviceId::new(0x046d, 0x0825, None)
}

fn set(devices: &[UsbDeviceId]) -> BTreeSet<UsbDeviceId> {
    devices.iter().cloned().collect()
}

/// What the switch moves: the keyboard and the mouse.
fn learned() -> Follower {
    Follower::new(set(&[keyboard(), mouse()]))
}

/// Shows the follower `present` for `polls` reads in a row; how many fired.
fn fires(follower: &mut Follower, present: &[UsbDeviceId], polls: u32, follow: Follow) -> usize {
    let present = set(present);
    (0..polls)
        .filter_map(|_| follower.observe(&present, follow))
        .count()
}

#[test]
fn fires_once_when_all_learned_devices_left() {
    let mut follower = learned();
    assert_eq!(
        fires(
            &mut follower,
            &[keyboard(), mouse(), webcam()],
            2,
            Follow::On
        ),
        0
    );
    assert_eq!(follower.state(), FollowerState::Armed);

    assert_eq!(fires(&mut follower, &[webcam()], LEAVE - 1, Follow::On), 0);
    assert_eq!(follower.observe(&set(&[webcam()]), Follow::On), Some(Fire));
    assert_eq!(
        POLL_INTERVAL * LEAVE,
        Duration::from_millis(1500),
        "about 1.5 s of absence"
    );

    assert_eq!(follower.state(), FollowerState::Spent);
    assert_eq!(fires(&mut follower, &[webcam()], 20, Follow::On), 0);
}

#[test]
fn arrival_never_fires() {
    let mut follower = learned();
    assert_eq!(fires(&mut follower, &[webcam()], 5, Follow::On), 0);
    assert_eq!(fires(&mut follower, &[keyboard()], 5, Follow::On), 0);
    assert_eq!(
        fires(&mut follower, &[keyboard(), mouse()], 5, Follow::On),
        0
    );

    assert_eq!(fires(&mut follower, &[], LEAVE, Follow::On), 1);
    assert_eq!(fires(&mut follower, &[mouse()], 5, Follow::On), 0);
    assert_eq!(
        fires(&mut follower, &[keyboard(), mouse()], 5, Follow::On),
        0
    );
    assert_eq!(follower.state(), FollowerState::Armed);
}

#[test]
fn disabled_never_fires() {
    let mut follower = learned();
    assert_eq!(
        fires(&mut follower, &[keyboard(), mouse()], 2, Follow::Off),
        0
    );
    assert_eq!(fires(&mut follower, &[], 20, Follow::Off), 0);

    // The leave seen while off is spent: turning on later does not replay it.
    assert_eq!(follower.state(), FollowerState::Spent);
    assert_eq!(fires(&mut follower, &[], 20, Follow::On), 0);
}

#[test]
fn partial_leave_does_not_fire() {
    let mut follower = learned();
    assert_eq!(
        fires(&mut follower, &[keyboard(), mouse()], 2, Follow::On),
        0
    );

    assert_eq!(fires(&mut follower, &[mouse()], 20, Follow::On), 0);
    assert_eq!(
        fires(&mut follower, &[keyboard(), webcam()], 20, Follow::On),
        0
    );
    assert_eq!(follower.state(), FollowerState::Armed);
}

#[test]
fn absence_shorter_than_debounce_does_not_fire() {
    let mut follower = learned();
    assert_eq!(
        fires(&mut follower, &[keyboard(), mouse()], 1, Follow::On),
        0
    );

    for _ in 0..5 {
        assert_eq!(fires(&mut follower, &[], LEAVE - 1, Follow::On), 0);
        assert_eq!(fires(&mut follower, &[mouse()], 1, Follow::On), 0);
    }
    assert_eq!(follower.state(), FollowerState::Armed);
}

#[test]
fn fires_again_only_after_a_device_returned() {
    let mut follower = learned();
    assert_eq!(
        fires(&mut follower, &[keyboard(), mouse()], 1, Follow::On),
        0
    );
    assert_eq!(fires(&mut follower, &[], LEAVE, Follow::On), 1);
    assert_eq!(fires(&mut follower, &[webcam()], 50, Follow::On), 0);

    assert_eq!(fires(&mut follower, &[mouse()], 1, Follow::On), 0);
    assert_eq!(follower.state(), FollowerState::Armed);
    assert_eq!(fires(&mut follower, &[], LEAVE, Follow::On), 1);
    assert_eq!(fires(&mut follower, &[], 50, Follow::On), 0);
}

#[test]
fn all_absent_at_start_does_not_fire() {
    let mut follower = learned();
    assert_eq!(fires(&mut follower, &[webcam()], 20, Follow::On), 0);
    assert_eq!(follower.state(), FollowerState::Waiting);

    let mut nothing_learned = Follower::new(BTreeSet::new());
    assert_eq!(
        fires(&mut nothing_learned, &[keyboard(), mouse()], 5, Follow::On),
        0
    );
    assert_eq!(fires(&mut nothing_learned, &[], 20, Follow::On), 0);
    assert_eq!(nothing_learned.state(), FollowerState::Waiting);
}

#[test]
fn learn_is_present_before_minus_present_after() {
    let before = set(&[keyboard(), mouse(), webcam()]);
    let plugged_meanwhile = UsbDeviceId::new(0x0781, 0x5581, Some("STICK".to_owned()));
    let after = set(&[webcam(), plugged_meanwhile]);

    assert_eq!(learn(&before, &after), set(&[keyboard(), mouse()]));
    assert_eq!(learn(&before, &before), BTreeSet::new());
    assert_eq!(learn(&BTreeSet::new(), &after), BTreeSet::new());
}
