use ddc_adapters::{BackendCall, InMemoryMonitorBackend};
use ddc_core::domain::{DdcError, MonitorId, VcpCode, VcpValue};

use super::{
    Push, SCROLL_STEP_PERCENT, WHEEL_NOTCH, WheelQueue, drain_wheel, percent_of, scroll_brightness,
    scrolled_percent,
};
use crate::dto::{ErrorKind, UiError};
use crate::fixture::{RTK_ID, rtk_id, rtk_monitor};
use crate::panel::BrightnessChange;
use crate::panel::tests::{DELL_ID, Osd, dell_monitor, osd_with};

type Batch = Result<Option<BrightnessChange>, UiError>;

fn brightness_writes(backend: &InMemoryMonitorBackend) -> Vec<BackendCall> {
    backend
        .calls()
        .into_iter()
        .filter(|call| matches!(call, BackendCall::WriteVcp(..)))
        .collect()
}

fn change(id: &str, before: u16, after: u16) -> BrightnessChange {
    BrightnessChange {
        monitor_id: MonitorId::new(id),
        before,
        after,
    }
}

fn written(id: &str, before: u16, after: u16) -> Batch {
    Ok(Some(change(id, before, after)))
}

/// A queue holding `notches` whole notches, as the wheel would leave it.
fn queue_with(notches: i32) -> WheelQueue {
    let queue = WheelQueue::new();
    for _ in 0..notches.unsigned_abs() {
        queue.push(WHEEL_NOTCH * notches.signum());
    }
    queue
}

fn drained(osd: &Osd, queue: &WheelQueue) -> Vec<Batch> {
    let mut batches = Vec::new();
    drain_wheel(osd, None, queue, |batch| batches.push(batch));
    batches
}

#[test]
fn scroll_steps_five_percent_per_notch_of_120() {
    assert_eq!(SCROLL_STEP_PERCENT, 5);
    assert_eq!(WHEEL_NOTCH, 120);
}

#[test]
fn scroll_one_notch_up_adds_five_percent() {
    assert_eq!(scrolled_percent(50, 1), 55);
    assert_eq!(scrolled_percent(0, 1), 5);
    assert_eq!(scrolled_percent(73, 3), 88);
}

#[test]
fn scroll_one_notch_down_takes_five_percent() {
    assert_eq!(scrolled_percent(50, -1), 45);
    assert_eq!(scrolled_percent(100, -1), 95);
    assert_eq!(scrolled_percent(73, -3), 58);
}

#[test]
fn scroll_stops_at_zero_and_at_one_hundred() {
    assert_eq!(scrolled_percent(98, 1), 100);
    assert_eq!(scrolled_percent(100, 1), 100);
    assert_eq!(scrolled_percent(3, -1), 0);
    assert_eq!(scrolled_percent(0, -1), 0);
    assert_eq!(scrolled_percent(50, i32::MAX), 100);
    assert_eq!(scrolled_percent(50, i32::MIN), 0);
}

#[test]
fn scroll_reads_a_value_as_a_rounded_percentage_of_its_maximum() {
    let percent = |current, max| percent_of(VcpValue { current, max });

    assert_eq!(percent(75, 100), 75);
    assert_eq!(percent(191, 255), 75);
    assert_eq!(percent(1, 3), 33);
    assert_eq!(percent(2, 3), 67);
    assert_eq!(percent(0, 100), 0);
    assert_eq!(percent(120, 100), 100, "a reading above its maximum");
    assert_eq!(percent(5, 0), 0, "a zero maximum");
}

#[test]
fn scroll_wheel_parts_add_up_to_whole_notches() {
    let queue = WheelQueue::new();

    assert_eq!(queue.push(60), Push::Queued);
    assert_eq!(queue.push(30), Push::Queued);
    assert_eq!(queue.push(30), Push::Start);

    assert_eq!(queue.take(), Some(1));
    assert_eq!(queue.take(), None);
}

#[test]
fn scroll_a_delta_of_several_notches_counts_them_all_and_keeps_the_rest() {
    let queue = WheelQueue::new();

    assert_eq!(queue.push(3 * WHEEL_NOTCH + 40), Push::Start);
    assert_eq!(queue.take(), Some(3));
    assert_eq!(
        queue.push(80),
        Push::Queued,
        "40 + 80 is one more notch, for the writer"
    );
    assert_eq!(queue.take(), Some(1));
    assert_eq!(queue.take(), None);
}

#[test]
fn scroll_turning_the_other_way_drops_the_part_of_a_notch_sent() {
    let queue = WheelQueue::new();
    queue.push(100);

    assert_eq!(queue.push(-WHEEL_NOTCH), Push::Start);

    assert_eq!(queue.take(), Some(-1));
}

#[test]
fn scroll_notches_up_and_down_cancel_out_and_write_nothing() {
    let queue = WheelQueue::new();

    assert_eq!(queue.push(WHEEL_NOTCH), Push::Start);
    assert_eq!(queue.push(-WHEEL_NOTCH), Push::Queued);

    assert_eq!(queue.take(), None);
}

#[test]
fn scroll_starts_one_writer_for_a_burst_and_another_once_it_stopped() {
    let queue = WheelQueue::new();

    let pushes: Vec<Push> = (0..4).map(|_| queue.push(WHEEL_NOTCH)).collect();

    assert_eq!(
        pushes,
        [Push::Start, Push::Queued, Push::Queued, Push::Queued]
    );
    assert_eq!(queue.take(), Some(4));
    assert_eq!(
        queue.push(WHEEL_NOTCH),
        Push::Queued,
        "the writer still runs"
    );
    assert_eq!(queue.take(), Some(1));
    assert_eq!(queue.take(), None);
    assert_eq!(queue.push(WHEEL_NOTCH), Push::Start, "the writer stopped");
}

#[test]
fn scroll_discarded_notches_are_dropped_and_the_next_one_starts_a_writer() {
    let queue = queue_with(3);

    queue.discard();

    assert_eq!(queue.take(), None);
    assert_eq!(queue.push(WHEEL_NOTCH), Push::Start);
}

#[test]
fn scroll_a_burst_of_notches_becomes_one_write() {
    let (osd, backend) = osd_with([rtk_monitor()]);
    let queue = queue_with(3);

    let batches = drained(&osd, &queue);

    assert_eq!(batches, [written(RTK_ID, 75, 90)]);
    assert_eq!(
        brightness_writes(&backend),
        [BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 90)]
    );
    assert_eq!(queue.push(WHEEL_NOTCH), Push::Start, "the writer stopped");
}

#[test]
fn scroll_notches_that_arrive_during_a_write_become_the_next_single_write() {
    let (osd, backend) = osd_with([rtk_monitor()]);
    let queue = queue_with(-1);
    let mut batches = Vec::new();

    drain_wheel(&osd, None, &queue, |batch| {
        if batches.is_empty() {
            for _ in 0..4 {
                assert_eq!(queue.push(-WHEEL_NOTCH), Push::Queued);
            }
        }
        batches.push(batch);
    });

    assert_eq!(batches, [written(RTK_ID, 75, 70), written(RTK_ID, 70, 50)]);
    assert_eq!(
        brightness_writes(&backend),
        [
            BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 70),
            BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 50),
        ]
    );
}

#[test]
fn scroll_answers_what_the_monitor_kept_not_what_was_asked() {
    let monitor = rtk_monitor().ignoring_writes_to(VcpCode::BRIGHTNESS);
    let (osd, backend) = osd_with([monitor]);

    let outcome = scroll_brightness(&osd, &rtk_id(), 1);

    assert_eq!(outcome, Ok(Some(change(RTK_ID, 75, 75))));
    assert_eq!(
        brightness_writes(&backend),
        [BackendCall::WriteVcp(rtk_id(), VcpCode::BRIGHTNESS, 80)]
    );
}

#[test]
fn scroll_up_at_the_top_writes_nothing() {
    let (osd, backend) = osd_with([dell_monitor(100, 100)]);
    let queue = queue_with(2);

    let batches = drained(&osd, &queue);

    assert_eq!(batches, [Ok(None)]);
    assert_eq!(brightness_writes(&backend), []);
}

#[test]
fn scroll_down_at_the_bottom_writes_nothing() {
    let (osd, backend) = osd_with([dell_monitor(0, 100)]);

    let outcome = scroll_brightness(&osd, &MonitorId::new(DELL_ID), -1);

    assert_eq!(outcome, Ok(None));
    assert_eq!(brightness_writes(&backend), []);
}

#[test]
fn scroll_scales_to_a_maximum_other_than_100_and_answers_the_read_back() {
    let (osd, backend) = osd_with([dell_monitor(191, 255)]);

    let outcome = scroll_brightness(&osd, &MonitorId::new(DELL_ID), 1);

    assert_eq!(outcome, Ok(Some(change(DELL_ID, 191, 204))));
    assert_eq!(
        brightness_writes(&backend),
        [BackendCall::WriteVcp(
            MonitorId::new(DELL_ID),
            VcpCode::BRIGHTNESS,
            204
        )]
    );
}

#[test]
fn scroll_moves_the_selected_monitor_and_names_it() {
    let (osd, backend) = osd_with([rtk_monitor(), dell_monitor(30, 100)]);
    let queue = queue_with(1);
    let mut batches = Vec::new();

    drain_wheel(&osd, Some(MonitorId::new(DELL_ID)), &queue, |batch| {
        batches.push(batch);
    });

    assert_eq!(batches, [written(DELL_ID, 30, 35)]);
    assert_eq!(
        brightness_writes(&backend),
        [BackendCall::WriteVcp(
            MonitorId::new(DELL_ID),
            VcpCode::BRIGHTNESS,
            35
        )]
    );
}

#[test]
fn scroll_without_a_monitor_writes_nothing_and_stops_the_writer() {
    let (osd, backend) = osd_with([]);
    let queue = queue_with(2);

    let batches = drained(&osd, &queue);

    assert_eq!(
        batches,
        [Err(UiError {
            kind: ErrorKind::NotFound,
            message: "no monitor is reachable".to_owned(),
        })]
    );
    assert_eq!(brightness_writes(&backend), []);
    assert_eq!(queue.push(WHEEL_NOTCH), Push::Start);
}

#[test]
fn scroll_on_a_monitor_that_does_not_answer_writes_nothing_and_stops_the_writer() {
    let monitor = rtk_monitor().with_vcp_failure(VcpCode::BRIGHTNESS, DdcError::Timeout);
    let (osd, backend) = osd_with([monitor]);
    let queue = queue_with(1);

    let batches = drained(&osd, &queue);

    assert_eq!(
        batches,
        [Err(UiError {
            kind: ErrorKind::Timeout,
            message: "monitor did not respond in time".to_owned(),
        })]
    );
    assert_eq!(brightness_writes(&backend), []);
    assert_eq!(queue.push(WHEEL_NOTCH), Push::Start);
}
