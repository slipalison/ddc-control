use std::time::Duration;

use super::retry::RetryPolicies;
use super::{DdcHiBudgets, DdcHiMonitorBackend};

#[test]
fn ddc_hi_backend_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<DdcHiMonitorBackend>();
}

#[test]
fn default_budgets_follow_the_measured_operation_costs() {
    assert_eq!(
        DdcHiBudgets::default(),
        DdcHiBudgets {
            vcp: Duration::from_secs(1),
            capabilities: Duration::from_secs(8),
            enumerate: Duration::from_secs(5),
        }
    );
}

/// Only builds and drops the backend: construction touches no monitor, so
/// this is safe without hardware.
#[test]
fn backend_starts_and_stops_without_touching_a_monitor() {
    let budgets = DdcHiBudgets {
        vcp: Duration::from_millis(500),
        ..DdcHiBudgets::default()
    };

    let backend = DdcHiMonitorBackend::new().unwrap().with_budgets(budgets);

    assert!(format!("{backend:?}").contains("500ms"));
}

/// The backends the app and the CLI build: `new`, with and without
/// `with_budgets`. Building one only starts the worker thread, which touches
/// no monitor before its first call.
fn production_backends() -> Vec<DdcHiMonitorBackend> {
    let budgets = DdcHiBudgets {
        vcp: Duration::from_millis(500),
        ..DdcHiBudgets::default()
    };
    vec![
        DdcHiMonitorBackend::new().unwrap(),
        DdcHiMonitorBackend::new().unwrap().with_budgets(budgets),
    ]
}

/// The retry policies are no parameter of the public constructors, so the
/// wiring is the only place the settling of an input switch can be lost: a
/// client given a zero settle window answers `Ok` right after the write, and
/// every test of the worker stays green (D-2026-09-30-input-switch-autostart-19).
#[test]
fn the_real_backend_hands_the_default_retry_policies_to_its_client() {
    for backend in production_backends() {
        assert_eq!(
            backend.client.policies,
            RetryPolicies::default(),
            "{backend:?}"
        );
    }
}
