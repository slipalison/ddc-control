//! Places the popup next to the tray icon on KDE Plasma under Wayland
//! (D-2026-09-27-tray-app-3). Wayland lets no app place its own window, so
//! the app hands KWin a script (`kwin/anchor.js`) over its D-Bus scripting
//! interface: each time the popup is mapped, the script moves it next to the
//! pointer — the icon or menu item it was opened from — inside the work
//! area of that screen. The script is loaded at start and unloaded when the
//! app quits, through its **Quit** item or a stop signal (SIGTERM, SIGINT,
//! SIGHUP); one left by a SIGKILL or a crash only matches its own process
//! id, and the next start replaces it.
//!
//! Anywhere else — X11, another desktop, a KWin without scripting — nothing
//! is loaded and the popup opens where the compositor puts it; a failure is
//! only a diagnostic line.

use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use zbus::Connection;

use crate::{diagnose, report};

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

/// How long the app waits for D-Bus while it quits: a KWin that does not
/// answer must not keep the app from quitting.
const QUIT_TIMEOUT: Duration = Duration::from_secs(2);

/// A script the app loaded into KWin, to unload as it quits.
pub(super) struct LoadedScript {
    path: PathBuf,
}

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

/// Loads the script into KWin when the session takes it, and answers what
/// it loaded.
pub(super) async fn install() -> Option<LoadedScript> {
    let session_type = std::env::var_os("XDG_SESSION_TYPE");
    let desktop = std::env::var_os("XDG_CURRENT_DESKTOP");
    if !applies(session_type.as_deref(), desktop.as_deref()) {
        return None;
    }
    let Some(path) = script_path(std::env::var_os("XDG_RUNTIME_DIR").as_deref()) else {
        diagnose("popup placement unavailable: no XDG_RUNTIME_DIR to write its script to");
        return None;
    };
    match load(&path).await {
        Ok(id) => {
            diagnose(&format!("popup placement loaded into KWin as script {id}"));
            Some(LoadedScript { path })
        }
        Err(error) => {
            diagnose(&format!("popup placement unavailable: {error}"));
            None
        }
    }
}

/// Unloads `script` from KWin and removes its file, as the app quits,
/// waiting at most [`QUIT_TIMEOUT`] for D-Bus.
pub(super) async fn uninstall(script: &LoadedScript) {
    let action = "unload the popup placement from KWin";
    match tokio::time::timeout(QUIT_TIMEOUT, unload_from_session()).await {
        Ok(Ok(true)) => diagnose("popup placement unloaded from KWin"),
        Ok(Ok(false)) => diagnose("popup placement was no longer loaded in KWin"),
        Ok(Err(error)) => report(action, &error),
        Err(_) => report(action, &format!("no answer within {QUIT_TIMEOUT:?}")),
    }
    discard(&script.path);
}

/// Removes the script's file, which only KWin reads. A failure is only
/// reported: the file lies in the user's runtime directory, which the
/// session empties at logout, and the next start overwrites it.
fn discard(path: &Path) {
    if let Err(error) = remove_script(path) {
        report("remove the popup placement script", &error);
    }
}

/// Removes the file at `path`; one already gone counts as removed.
fn remove_script(path: &Path) -> io::Result<()> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

/// Writes the script to `path` and has KWin load and run it; a script KWin
/// did not take is not left behind.
async fn load(path: &Path) -> Result<i32, String> {
    std::fs::write(path, script_for(std::process::id())).map_err(|error| error.to_string())?;
    let loaded = load_written(path).await;
    if loaded.is_err() {
        discard(path);
    }
    loaded
}

async fn load_written(path: &Path) -> Result<i32, String> {
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

async fn unload_from_session() -> zbus::Result<bool> {
    unload(&Connection::session().await?).await
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
    use std::path::{Path, PathBuf};

    use super::{PID_MARK, SCRIPT, applies, remove_script, script_for, script_path};

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

    /// A path of its own in the temporary directory, for one test.
    fn scratch(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("ddc-tray-{}-{name}", std::process::id()))
    }

    #[test]
    fn removing_the_script_removes_its_file_and_a_gone_one_is_fine() {
        let path = scratch("remove-script.js");
        std::fs::write(&path, "script").unwrap();

        assert!(remove_script(&path).is_ok());
        assert!(!path.exists());
        assert!(remove_script(&path).is_ok(), "already gone");
    }

    #[test]
    fn a_script_that_cannot_be_removed_is_an_error() {
        let path = scratch("remove-script-dir");
        std::fs::create_dir(&path).unwrap();

        let removed = remove_script(&path);

        std::fs::remove_dir(&path).unwrap();
        assert!(removed.is_err());
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
