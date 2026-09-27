//! Places the popup next to the tray icon on KDE Plasma under Wayland
//! (D-2026-09-27-tray-app-3). Wayland lets no app place its own window, so
//! the app hands KWin a script (`kwin/anchor.js`) over its D-Bus scripting
//! interface: each time the popup is mapped, the script moves it next to the
//! pointer — the icon or menu item it was opened from — inside the work
//! area of that screen. The script is loaded at start and unloaded when the
//! app quits; one left by a crash only matches its own process id, and the
//! next start replaces it.
//!
//! Anywhere else — X11, another desktop, a KWin without scripting — nothing
//! is loaded and the popup opens where the compositor puts it; a failure is
//! only a diagnostic line.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use zbus::Connection;

use crate::diagnose;

/// The script, with [`PID_MARK`] where the app's process id goes.
const SCRIPT: &str = include_str!("../../kwin/anchor.js");

/// The placeholder of the script for the process id of the popup.
const PID_MARK: &str = "__PID__";

/// The name the script is loaded under in KWin.
const PLUGIN: &str = "ddc-tray-anchor";

/// The file the script is written to, in the user's runtime directory.
const SCRIPT_FILE: &str = "ddc-tray-kwin-anchor.js";

const KWIN: &str = "org.kde.KWin";
const SCRIPTING_PATH: &str = "/Scripting";
const SCRIPTING: &str = "org.kde.kwin.Scripting";
const SCRIPT_INTERFACE: &str = "org.kde.kwin.Script";

/// Whether the session can take the script: Wayland under KDE Plasma, as
/// `XDG_SESSION_TYPE` and `XDG_CURRENT_DESKTOP` say.
pub(super) fn applies(session_type: Option<&OsStr>, desktop: Option<&OsStr>) -> bool {
    let wayland = session_type.is_some_and(|session| session.eq_ignore_ascii_case("wayland"));
    let plasma = desktop.and_then(OsStr::to_str).is_some_and(|desktops| {
        desktops
            .split(':')
            .any(|name| name.eq_ignore_ascii_case("KDE"))
    });
    wayland && plasma
}

/// The script for the popup of process `pid`.
pub(super) fn script_for(pid: u32) -> String {
    SCRIPT.replace(PID_MARK, &pid.to_string())
}

/// Where the script is written for KWin to read: the user's runtime
/// directory, `runtime_dir` (`XDG_RUNTIME_DIR`), which only the user can
/// write to. None without one, or with one that is not an absolute path:
/// in a shared directory such as `/tmp`, another local user could swap the
/// code KWin runs in the session, so the popup opens unplaced instead.
pub(super) fn script_path(runtime_dir: Option<&OsStr>) -> Option<PathBuf> {
    let dir = Path::new(runtime_dir?);
    dir.is_absolute().then(|| dir.join(SCRIPT_FILE))
}

/// Loads the script into KWin when the session takes it.
pub(super) async fn install() {
    let session_type = std::env::var_os("XDG_SESSION_TYPE");
    let desktop = std::env::var_os("XDG_CURRENT_DESKTOP");
    if !applies(session_type.as_deref(), desktop.as_deref()) {
        return;
    }
    let Some(path) = script_path(std::env::var_os("XDG_RUNTIME_DIR").as_deref()) else {
        diagnose("popup placement unavailable: no XDG_RUNTIME_DIR to write its script to");
        return;
    };
    match load(&path).await {
        Ok(id) => diagnose(&format!("popup placement loaded into KWin as script {id}")),
        Err(error) => diagnose(&format!("popup placement unavailable: {error}")),
    }
}

/// Unloads the script, if one is loaded under the app's name.
pub(super) async fn uninstall() {
    let Ok(connection) = Connection::session().await else {
        return;
    };
    if unload(&connection).await.unwrap_or(false) {
        diagnose("popup placement unloaded from KWin");
    }
    if let Some(path) = script_path(std::env::var_os("XDG_RUNTIME_DIR").as_deref()) {
        let _ = std::fs::remove_file(path);
    }
}

async fn load(path: &Path) -> Result<i32, String> {
    std::fs::write(path, script_for(std::process::id())).map_err(|error| error.to_string())?;
    let connection = Connection::session()
        .await
        .map_err(|error| error.to_string())?;
    // A script a crashed run left under the same name would block the load.
    unload(&connection)
        .await
        .map_err(|error| error.to_string())?;
    let id: i32 = connection
        .call_method(
            Some(KWIN),
            SCRIPTING_PATH,
            Some(SCRIPTING),
            "loadScript",
            &(path.to_string_lossy().as_ref(), PLUGIN),
        )
        .await
        .and_then(|reply| reply.body().deserialize())
        .map_err(|error| error.to_string())?;
    if id < 0 {
        return Err(format!("KWin refused the script ({id})"));
    }
    connection
        .call_method(
            Some(KWIN),
            format!("{SCRIPTING_PATH}/Script{id}").as_str(),
            Some(SCRIPT_INTERFACE),
            "run",
            &(),
        )
        .await
        .map_err(|error| error.to_string())?;
    Ok(id)
}

async fn unload(connection: &Connection) -> zbus::Result<bool> {
    connection
        .call_method(
            Some(KWIN),
            SCRIPTING_PATH,
            Some(SCRIPTING),
            "unloadScript",
            &(PLUGIN,),
        )
        .await?
        .body()
        .deserialize()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use super::{PID_MARK, SCRIPT, applies, script_for, script_path};

    fn os(value: &str) -> Option<&OsStr> {
        Some(OsStr::new(value))
    }

    #[test]
    fn the_placement_applies_to_plasma_under_wayland_only() {
        assert!(applies(os("wayland"), os("KDE")));
        assert!(applies(os("Wayland"), os("kde")));
        assert!(applies(os("wayland"), os("ubuntu:KDE")));
        for (session, desktop) in [
            (os("x11"), os("KDE")),
            (os("wayland"), os("GNOME")),
            (os("wayland"), os("KDEX")),
            (None, os("KDE")),
            (os("wayland"), None),
            (os("tty"), os("")),
        ] {
            assert!(!applies(session, desktop), "{session:?} {desktop:?}");
        }
    }

    #[test]
    fn the_script_goes_to_the_user_s_runtime_directory() {
        assert_eq!(
            script_path(os("/run/user/1000")).as_deref(),
            Some(Path::new("/run/user/1000/ddc-tray-kwin-anchor.js"))
        );
    }

    #[test]
    fn without_a_runtime_directory_no_script_is_written_anywhere() {
        for runtime_dir in [None, os(""), os("run/user/1000"), os("./tmp")] {
            assert_eq!(script_path(runtime_dir), None, "{runtime_dir:?}");
        }
    }

    #[test]
    fn the_script_looks_for_the_class_and_title_the_popup_has() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../../tauri.conf.json")).unwrap();

        assert_eq!(config["app"]["windows"][0]["label"], "popup");
        assert_eq!(config["app"]["windows"][0]["title"], "DDC Control");
        assert_eq!(
            env!("CARGO_PKG_NAME"),
            "ddc-tray",
            "the binary, whose name is the class"
        );
    }

    #[test]
    fn the_script_matches_the_popup_of_the_app_s_own_process_only() {
        assert_eq!(SCRIPT.matches(PID_MARK).count(), 1);

        let script = script_for(4242);

        assert!(script.contains("const POPUP_PID = 4242;"));
        assert!(!script.contains(PID_MARK));
        assert!(script.contains("const POPUP_CLASS = \"ddc-tray\";"));
        assert!(script.contains("const POPUP_CAPTION = \"DDC Control\";"));
    }
}
