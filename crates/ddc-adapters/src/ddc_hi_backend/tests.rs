use std::time::Duration;

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
