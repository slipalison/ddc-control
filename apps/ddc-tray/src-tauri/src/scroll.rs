//! The mouse wheel over the tray icon (D-2026-09-27-tray-app-2): each notch
//! moves the brightness of the tray's target monitor by
//! [`SCROLL_STEP_PERCENT`] of its maximum — up for a notch away from the
//! user — never past 0 or 100 %. Notches that arrive while a write runs
//! wait and become one write: DDC/CI is slow, and a monitor takes one
//! transaction at a time.
//!
//! Plain code on the core's port: [`WheelQueue`] turns the wheel's deltas
//! into batches of notches, and [`drain_wheel`] writes them. The tray only
//! feeds the one and runs the other off the main thread.

use std::sync::{Mutex, MutexGuard, PoisonError};

use ddc_core::domain::{Confirm, MonitorId, VcpCode, VcpValue};
use ddc_core::ports::MonitorControl;

use crate::dto::{PanelChangedDto, UiError};
use crate::panel::{brightness_for_percent, shortcut_target, ui_error};

/// How far one notch moves the brightness, in percent of its maximum.
pub const SCROLL_STEP_PERCENT: u8 = 5;

/// The delta of one wheel notch, as Plasma sends it (Qt's `angleDelta`,
/// positive away from the user). Finer wheels and touchpads send parts of
/// it, which add up.
pub const WHEEL_NOTCH: i32 = 120;

/// What a wheel delta asks of the tray.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Push {
    /// Start a writer: notches wait and no writer runs.
    Start,
    /// Nothing: the running writer takes the notches, or the delta did not
    /// complete a notch.
    Queued,
}

#[derive(Debug, Default)]
struct Pending {
    /// The part of a notch the wheel sent so far.
    remainder: i32,
    /// Whole notches not yet written.
    notches: i32,
    /// Whether a writer runs.
    writing: bool,
}

/// The notches the wheel asked for and no write has taken yet, shared by
/// the tray's scroll handler and the writer it starts.
#[derive(Debug, Default)]
pub struct WheelQueue {
    pending: Mutex<Pending>,
}

impl WheelQueue {
    /// An empty queue, with no writer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a wheel `delta`. A turn the other way drops the part of a
    /// notch the wheel had sent, so reversing always moves at once.
    pub fn push(&self, delta: i32) -> Push {
        let mut pending = self.lock();
        if delta.signum() * pending.remainder.signum() < 0 {
            pending.remainder = 0;
        }
        let total = pending.remainder.saturating_add(delta);
        pending.remainder = total % WHEEL_NOTCH;
        pending.notches = pending.notches.saturating_add(total / WHEEL_NOTCH);
        if pending.notches == 0 || pending.writing {
            return Push::Queued;
        }
        pending.writing = true;
        Push::Start
    }

    /// The notches waiting, for the writer. `None` once none waits: the
    /// writer stops, and the next notch starts another.
    pub fn take(&self) -> Option<i32> {
        let mut pending = self.lock();
        let notches = std::mem::take(&mut pending.notches);
        if notches == 0 {
            pending.writing = false;
            return None;
        }
        Some(notches)
    }

    /// Drops the notches waiting and stops the writer — the core could not
    /// be reached at all.
    pub fn discard(&self) {
        let mut pending = self.lock();
        pending.notches = 0;
        pending.writing = false;
    }

    /// A poisoned lock only means a handler panicked mid-update; the counts
    /// it guards are still meaningful.
    fn lock(&self) -> MutexGuard<'_, Pending> {
        self.pending.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The percentage of its maximum `value` stands for, rounded; 0 when the
/// maximum is 0.
pub fn percent_of(value: VcpValue) -> u8 {
    if value.max == 0 {
        return 0;
    }
    let max = u32::from(value.max);
    let percent = (u32::from(value.current) * 100 + max / 2) / max;
    u8::try_from(percent.min(100)).unwrap_or(100)
}

/// The brightness `notches` notches lead to from `percent`, kept within
/// 0–100 %.
pub fn scrolled_percent(percent: u8, notches: i32) -> u8 {
    let step = i32::from(SCROLL_STEP_PERCENT);
    let moved = i32::from(percent).saturating_add(notches.saturating_mul(step));
    u8::try_from(moved.clamp(0, 100)).unwrap_or(0)
}

/// Moves monitor `id`'s brightness by `notches`: one read for the current
/// value and its maximum, then one write — none when the brightness is
/// already at the end the wheel pushes to. Brightness is safe to write, so
/// no confirmation is given. Returns the value read back after a write.
///
/// # Errors
///
/// The [`UiError`] of the read or of the write.
pub fn scroll_brightness<M: MonitorControl + ?Sized>(
    osd: &M,
    id: &MonitorId,
    notches: i32,
) -> Result<Option<u16>, UiError> {
    let value = osd
        .get_feature(id, VcpCode::BRIGHTNESS)
        .map_err(ui_error)?
        .value;
    let target = brightness_for_percent(value.max, scrolled_percent(percent_of(value), notches));
    if target == value.current {
        return Ok(None);
    }
    osd.set_feature(id, VcpCode::BRIGHTNESS, target, Confirm::No)
        .map(|read_back| Some(read_back.current))
        .map_err(ui_error)
}

/// Writes what `wheel` holds, one batch of notches per write, until nothing
/// waits. The target is `selected`, else the first monitor listed.
/// `on_batch` hears each batch: the monitor written, `None` for a batch
/// that had nothing to write, or the error.
pub fn drain_wheel<M, F>(osd: &M, selected: Option<MonitorId>, wheel: &WheelQueue, mut on_batch: F)
where
    M: MonitorControl + ?Sized,
    F: FnMut(Result<Option<PanelChangedDto>, UiError>),
{
    let id = match shortcut_target(osd, selected) {
        Ok(id) => id,
        Err(error) => {
            wheel.discard();
            on_batch(Err(error));
            return;
        }
    };
    let changed = || PanelChangedDto {
        monitor_id: id.as_str().to_owned(),
    };
    while let Some(notches) = wheel.take() {
        on_batch(scroll_brightness(osd, &id, notches).map(|written| written.map(|_| changed())));
    }
}

#[cfg(test)]
mod tests;
