//! The popup's commands (D-2026-09-26-tray-app-4): thin `async` wrappers
//! that run the presentation of [`crate::panel`] on a blocking thread, plus
//! the synchronous pieces they share with the tray — the app state and the
//! write the user asked for.
//!
//! This is the one place where the user's "yes" from the popup's dialog
//! becomes [`Confirm::Yes`] (D-2026-09-26-tray-app-7).

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use ddc_core::domain::{Confirm, DdcError, MonitorId, VcpCode};
use ddc_core::ports::MonitorControl;
use tauri::{AppHandle, State, WebviewWindow};

use crate::dto::{ErrorKind, FeatureDto, MonitorDto, PanelDto, ReadBackDto, UiError};
use crate::panel::{self, ui_error};

/// The core as the commands share it, whatever backend it was built on.
pub type SharedOsd = Arc<dyn MonitorControl + Send + Sync>;

/// What every command shares: the core — or why it could not be built —,
/// the monitor the popup last selected and the monitors it last listed.
pub struct AppState {
    osd: Result<SharedOsd, UiError>,
    selected: Mutex<Option<MonitorId>>,
    listed: Mutex<Vec<MonitorDto>>,
}

impl AppState {
    /// The state over `osd`, the outcome of building the core. A backend
    /// that could not start makes every command answer
    /// `backend_unavailable`, and the app keeps running.
    pub fn new(osd: Result<SharedOsd, DdcError>) -> Self {
        Self {
            osd: osd.map_err(backend_unavailable),
            selected: Mutex::new(None),
            listed: Mutex::new(Vec::new()),
        }
    }

    /// The core.
    ///
    /// # Errors
    ///
    /// `backend_unavailable` when it could not be built.
    pub fn osd(&self) -> Result<SharedOsd, UiError> {
        self.osd.clone()
    }

    /// Remembers `id` as the monitor the popup shows.
    pub fn select(&self, id: MonitorId) {
        *self.selection() = Some(id);
    }

    /// The monitor the popup last selected, if any.
    pub fn selected(&self) -> Option<MonitorId> {
        self.selection().clone()
    }

    /// Remembers the monitors the popup was last given, for their names.
    pub fn remember_listed(&self, monitors: &[MonitorDto]) {
        *self.listing() = monitors.to_vec();
    }

    /// The name the picker shows for the selected monitor — the tray's
    /// tooltip names it —, once the popup selected one it listed.
    pub fn selected_label(&self) -> Option<String> {
        let selected = self.selected()?;
        self.listing()
            .iter()
            .find(|monitor| monitor.id == selected.as_str())
            .map(|monitor| monitor.label.clone())
    }

    /// A poisoned lock only means a command panicked mid-update; the id it
    /// guards is still meaningful.
    fn selection(&self) -> MutexGuard<'_, Option<MonitorId>> {
        self.selected.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Same as [`Self::selection`], for the monitors listed.
    fn listing(&self) -> MutexGuard<'_, Vec<MonitorDto>> {
        self.listed.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The UI's view of a backend that could not start.
pub fn backend_unavailable(error: DdcError) -> UiError {
    UiError {
        kind: ErrorKind::BackendUnavailable,
        message: error.to_string(),
    }
}

/// A write as the popup asks for it (A-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteRequest {
    /// The monitor to write to.
    pub monitor_id: MonitorId,
    /// The feature to write.
    pub code: VcpCode,
    /// The value asked for.
    pub value: u16,
    /// Whether the user confirmed the write in the popup's dialog; the UI
    /// sends `true` only after that dialog.
    pub confirmed: bool,
}

impl WriteRequest {
    fn confirmation(&self) -> Confirm {
        if self.confirmed {
            Confirm::Yes
        } else {
            Confirm::No
        }
    }
}

/// Performs `request` and returns the value read back right after — what
/// the monitor kept, never the value asked for. The core refuses a
/// dangerous write the user did not confirm before touching the monitor.
///
/// # Errors
///
/// The [`UiError`] of the core's refusal or of the write.
pub fn write_feature<M: MonitorControl + ?Sized>(
    osd: &M,
    request: &WriteRequest,
) -> Result<ReadBackDto, UiError> {
    osd.set_feature(
        &request.monitor_id,
        request.code,
        request.value,
        request.confirmation(),
    )
    .map(ReadBackDto::from)
    .map_err(ui_error)
}

/// Runs `call` on the core on a blocking thread: a DDC/CI round-trip takes
/// 50–200 ms and a probe several seconds, which must hold neither the main
/// thread nor the async runtime.
///
/// # Errors
///
/// `backend_unavailable` without a core, the error `call` returns, or
/// `transport` when `call` stopped before answering.
pub async fn on_blocking_thread<T, F>(state: &AppState, call: F) -> Result<T, UiError>
where
    T: Send + 'static,
    F: FnOnce(&(dyn MonitorControl + Send + Sync)) -> Result<T, UiError> + Send + 'static,
{
    let osd = state.osd()?;
    tauri::async_runtime::spawn_blocking(move || call(osd.as_ref()))
        .await
        .map_err(|stopped| UiError {
            kind: ErrorKind::Transport,
            message: format!("the monitor call stopped before answering: {stopped}"),
        })?
}

/// The reachable monitors.
#[tauri::command]
pub async fn list_monitors(state: State<'_, AppState>) -> Result<Vec<MonitorDto>, UiError> {
    let monitors = on_blocking_thread(&state, |osd| panel::monitors(osd)).await?;
    state.remember_listed(&monitors);
    Ok(monitors)
}

/// Remembers the monitor the popup shows, the target of tray shortcuts and
/// of the wheel over the icon, which the tray's tooltip names.
#[tauri::command]
pub async fn select_monitor(
    app: AppHandle,
    state: State<'_, AppState>,
    monitor_id: String,
) -> Result<(), UiError> {
    state.select(MonitorId::new(monitor_id));
    crate::tray::selection_changed(&app);
    Ok(())
}

/// The quick controls of a monitor.
#[tauri::command]
pub async fn load_panel(
    state: State<'_, AppState>,
    monitor_id: String,
) -> Result<PanelDto, UiError> {
    let id = MonitorId::new(monitor_id);
    on_blocking_thread(&state, move |osd| panel::load_panel(osd, &id)).await
}

/// The "all settings" entries the monitor's capabilities declare.
#[tauri::command]
pub async fn load_features(
    state: State<'_, AppState>,
    monitor_id: String,
) -> Result<Vec<FeatureDto>, UiError> {
    let id = MonitorId::new(monitor_id);
    on_blocking_thread(&state, move |osd| panel::load_features(osd, &id)).await
}

/// The "all settings" entries a probe of the undeclared codes finds.
#[tauri::command]
pub async fn probe_features(
    state: State<'_, AppState>,
    monitor_id: String,
) -> Result<Vec<FeatureDto>, UiError> {
    let id = MonitorId::new(monitor_id);
    on_blocking_thread(&state, move |osd| panel::probe_features(osd, &id)).await
}

/// Writes a feature and returns the value read back.
#[tauri::command]
pub async fn set_feature(
    state: State<'_, AppState>,
    monitor_id: String,
    code: u8,
    value: u16,
    confirmed: bool,
) -> Result<ReadBackDto, UiError> {
    let request = WriteRequest {
        monitor_id: MonitorId::new(monitor_id),
        code: VcpCode(code),
        value,
        confirmed,
    };
    on_blocking_thread(&state, move |osd| write_feature(osd, &request)).await
}

/// Hides the popup (Esc).
#[tauri::command]
pub async fn hide_popup(window: WebviewWindow) {
    match window.hide() {
        Ok(()) => crate::diagnose("popup hidden"),
        Err(error) => crate::report("hide the popup", &error),
    }
}

#[cfg(test)]
mod tests;
