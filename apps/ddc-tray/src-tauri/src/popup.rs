//! What a click on the tray icon does to the popup, given the blur that same
//! click may have caused (D-2026-09-26-tray-app-5).
//!
//! Clicking the icon while the popup is open first takes the focus away from
//! the popup, which hides on blur, and only then delivers the click: read
//! naively, that click would open the popup again right away. The gate
//! remembers when the popup last hid on blur and lets a click that follows
//! it closely do nothing. Pure: the caller passes the time in.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// How long after the popup hid on blur a tray click is taken as the click
/// that caused that blur.
pub const BLUR_CLICK_WINDOW: Duration = Duration::from_millis(300);

/// Whether the popup is on screen when the tray icon is clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// The popup is on screen.
    Shown,
    /// The popup is hidden.
    Hidden,
}

/// What a tray click does to the popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickAction {
    /// Show the popup and focus it.
    Show,
    /// Hide the popup.
    Hide,
    /// Leave the popup hidden: the click already closed it through the blur.
    Nothing,
}

/// Remembers when the popup last hid on blur, shared by the window's blur
/// handler and the tray's click handler.
#[derive(Debug, Default)]
pub struct PopupGate {
    hidden_on_blur_at: Mutex<Option<Instant>>,
}

impl PopupGate {
    /// A gate that has not seen the popup hide yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records that the popup hid at `at` because it lost the focus.
    pub fn hidden_on_blur(&self, at: Instant) {
        *self.last_blur() = Some(at);
    }

    /// What a tray click at `at` does to a popup that is `popup`: an open
    /// popup hides; a hidden one shows, unless it hid on blur at most
    /// [`BLUR_CLICK_WINDOW`] before.
    pub fn tray_clicked(&self, popup: Visibility, at: Instant) -> ClickAction {
        if popup == Visibility::Shown {
            return ClickAction::Hide;
        }
        match *self.last_blur() {
            Some(hidden) if at.saturating_duration_since(hidden) <= BLUR_CLICK_WINDOW => {
                ClickAction::Nothing
            }
            _ => ClickAction::Show,
        }
    }

    /// A poisoned lock only means a handler panicked mid-update; the instant
    /// it guards is still meaningful.
    fn last_blur(&self) -> MutexGuard<'_, Option<Instant>> {
        self.hidden_on_blur_at
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{BLUR_CLICK_WINDOW, ClickAction, PopupGate, Visibility};

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn the_blur_click_window_is_300_ms() {
        assert_eq!(BLUR_CLICK_WINDOW, ms(300));
    }

    #[test]
    fn a_click_on_a_hidden_popup_that_never_blurred_shows_it() {
        let gate = PopupGate::new();

        let action = gate.tray_clicked(Visibility::Hidden, Instant::now());

        assert_eq!(action, ClickAction::Show);
    }

    #[test]
    fn a_click_on_an_open_popup_hides_it() {
        let gate = PopupGate::new();
        let now = Instant::now();
        gate.hidden_on_blur(now);

        let action = gate.tray_clicked(Visibility::Shown, now + ms(10));

        assert_eq!(action, ClickAction::Hide);
    }

    #[test]
    fn a_click_right_after_a_blur_hide_does_not_reopen_the_popup() {
        let gate = PopupGate::new();
        let hidden = Instant::now();
        gate.hidden_on_blur(hidden);

        assert_eq!(
            gate.tray_clicked(Visibility::Hidden, hidden),
            ClickAction::Nothing
        );
        assert_eq!(
            gate.tray_clicked(Visibility::Hidden, hidden + ms(120)),
            ClickAction::Nothing
        );
        assert_eq!(
            gate.tray_clicked(Visibility::Hidden, hidden + ms(300)),
            ClickAction::Nothing
        );
    }

    #[test]
    fn a_click_after_the_window_shows_the_popup_again() {
        let gate = PopupGate::new();
        let hidden = Instant::now();
        gate.hidden_on_blur(hidden);

        let action = gate.tray_clicked(Visibility::Hidden, hidden + ms(301));

        assert_eq!(action, ClickAction::Show);
    }

    #[test]
    fn a_click_stamped_before_the_blur_counts_as_the_same_gesture() {
        let gate = PopupGate::new();
        let clicked = Instant::now();
        gate.hidden_on_blur(clicked + ms(5));

        let action = gate.tray_clicked(Visibility::Hidden, clicked);

        assert_eq!(action, ClickAction::Nothing);
    }

    #[test]
    fn only_the_latest_blur_hide_counts() {
        let gate = PopupGate::new();
        let first = Instant::now();
        gate.hidden_on_blur(first);
        gate.hidden_on_blur(first + ms(1_000));

        assert_eq!(
            gate.tray_clicked(Visibility::Hidden, first + ms(1_200)),
            ClickAction::Nothing
        );
        assert_eq!(
            gate.tray_clicked(Visibility::Hidden, first + ms(1_400)),
            ClickAction::Show
        );
    }
}
