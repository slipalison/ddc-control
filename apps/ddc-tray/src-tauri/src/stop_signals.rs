//! Quits the app on SIGTERM, SIGINT and SIGHUP the way its **Quit** menu
//! item does (`AppHandle::exit`): the exit event then runs, and the app
//! undoes what it set up outside itself — the KWin placement script — instead
//! of dying with it loaded. SIGTERM is what `pkill`, systemd and the session
//! send; SIGINT is Ctrl+C; SIGHUP a closed terminal. While the app quits, a
//! second stop signal exits at once, so a quit that hangs never needs a
//! SIGKILL.

use std::future::poll_fn;
use std::io;
use std::task::Poll;

use tauri::{AppHandle, Runtime};
use tokio::signal::unix::{Signal, SignalKind, signal};

use crate::{diagnose, report};

/// The signals that stop the app, with their names.
const STOP_SIGNALS: [(SignalKind, &str); 3] = [
    (SignalKind::terminate(), "SIGTERM"),
    (SignalKind::interrupt(), "SIGINT"),
    (SignalKind::hangup(), "SIGHUP"),
];

/// The exit status when a second stop signal cuts the quit short.
const EXIT_CUT_SHORT: i32 = 1;

/// Signals the app listens to; while it does, none of them kills it.
struct StopSignals(Vec<(Signal, &'static str)>);

impl StopSignals {
    /// Listens to `kinds`, from inside the async runtime.
    fn listen(kinds: &[(SignalKind, &'static str)]) -> io::Result<Self> {
        kinds
            .iter()
            .map(|&(kind, name)| Ok((signal(kind)?, name)))
            .collect::<io::Result<_>>()
            .map(Self)
    }

    /// The name of the next signal that arrives.
    async fn next(&mut self) -> &'static str {
        poll_fn(|context| {
            for (signal, name) in &mut self.0 {
                if let Poll::Ready(Some(())) = signal.poll_recv(context) {
                    return Poll::Ready(*name);
                }
            }
            Poll::Pending
        })
        .await
    }
}

/// From now on, a stop signal quits the app as its **Quit** item does.
pub(crate) fn quit_on_stop_signals<R: Runtime>(app: &AppHandle<R>) {
    let runtime = tauri::async_runtime::handle();
    // Listening here rather than in the task leaves no moment, once the app
    // is set up, when a stop signal still kills it outright.
    let listened = {
        let _inside = runtime.inner().enter();
        StopSignals::listen(&STOP_SIGNALS)
    };
    let mut signals = match listened {
        Ok(signals) => signals,
        Err(error) => {
            report("listen for SIGTERM, SIGINT and SIGHUP", &error);
            return;
        }
    };
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let first = signals.next().await;
        diagnose(&format!("{first} received, quitting"));
        app.exit(0);
        let second = signals.next().await;
        eprintln!("ddc-tray: {second} received while quitting, exiting at once");
        std::process::exit(EXIT_CUT_SHORT);
    });
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use tokio::signal::unix::SignalKind;

    use super::{STOP_SIGNALS, StopSignals};

    #[test]
    fn sigterm_sigint_and_sighup_stop_the_app() {
        assert_eq!(
            STOP_SIGNALS,
            [
                (SignalKind::terminate(), "SIGTERM"),
                (SignalKind::interrupt(), "SIGINT"),
                (SignalKind::hangup(), "SIGHUP"),
            ]
        );
    }

    #[test]
    fn a_stop_signal_the_app_listens_to_is_heard_by_name() {
        tauri::async_runtime::block_on(async {
            let mut signals = StopSignals::listen(&[(SignalKind::hangup(), "SIGHUP")]).unwrap();
            let pid = std::process::id().to_string();

            let sent = Command::new("kill").args(["-HUP", &pid]).status().unwrap();

            assert!(sent.success());
            assert_eq!(signals.next().await, "SIGHUP");
        });
    }
}
