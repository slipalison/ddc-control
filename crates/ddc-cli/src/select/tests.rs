use ddc_core::domain::{MonitorId, MonitorInfo};

use super::{SelectionError, select_monitor};

fn info(key: &str) -> MonitorInfo {
    MonitorInfo {
        id: MonitorId::new(key),
        manufacturer: None,
        model: None,
        serial: None,
    }
}

fn monitors(keys: &[&str]) -> Vec<MonitorInfo> {
    keys.iter().map(|key| info(key)).collect()
}

fn select(keys: &[&str], wanted: Option<&str>) -> Result<String, SelectionError> {
    select_monitor(&monitors(keys), wanted).map(|id| id.as_str().to_owned())
}

const DEV_BOX: [&str; 2] = ["RTK-RTK-QHD-HDR-01010101", "GSM-LG-TV-SSCR2-01010101"];

#[test]
fn no_monitor_is_an_error_with_or_without_the_flag() {
    assert_eq!(select(&[], None), Err(SelectionError::NoMonitor));
    assert_eq!(select(&[], Some("RTK")), Err(SelectionError::NoMonitor));
    assert_eq!(
        SelectionError::NoMonitor.to_string(),
        "no monitor found over DDC/CI"
    );
}

#[test]
fn the_only_monitor_is_selected_without_the_flag() {
    assert_eq!(select(&["ONLY-1"], None).unwrap(), "ONLY-1");
}

#[test]
fn several_monitors_without_the_flag_list_every_index_and_id() {
    let error = select(&DEV_BOX, None).unwrap_err();

    assert_eq!(
        error,
        SelectionError::ChoiceNeeded(vec![(1, info(DEV_BOX[0])), (2, info(DEV_BOX[1]))])
    );
    assert_eq!(
        error.to_string(),
        "2 monitors found; choose one with --monitor <id|index>:\n  \
         1  RTK-RTK-QHD-HDR-01010101\n  \
         2  GSM-LG-TV-SSCR2-01010101\n\
         e.g. --monitor 1 or --monitor RTK-RTK-QHD-HDR-01010101"
    );
}

#[test]
fn an_exact_id_wins_over_a_substring_of_another_id() {
    assert_eq!(select(&["ABC-2", "ABC"], Some("ABC")).unwrap(), "ABC");
    assert_eq!(select(&["ABC-2", "ABC"], Some("ABC-2")).unwrap(), "ABC-2");
}

#[test]
fn an_all_digit_value_is_a_one_based_index() {
    assert_eq!(select(&DEV_BOX, Some("1")).unwrap(), DEV_BOX[0]);
    assert_eq!(select(&DEV_BOX, Some("2")).unwrap(), DEV_BOX[1]);
    assert_eq!(select(&DEV_BOX, Some("02")).unwrap(), DEV_BOX[1]);
}

#[test]
fn an_index_out_of_range_never_falls_back_to_matching_ids() {
    let keys = ["MON-2", "MON-3"];
    for wanted in ["0", "3", "99999999999999999999999"] {
        let error = select(&keys, Some(wanted)).unwrap_err();

        assert_eq!(
            error,
            SelectionError::NoMatch {
                wanted: wanted.to_owned(),
                monitors: vec![(1, info("MON-2")), (2, info("MON-3"))],
            },
            "--monitor {wanted}"
        );
    }
}

#[test]
fn an_exact_all_digit_id_is_matched_as_an_id_first() {
    assert_eq!(select(&["A", "B", "1"], Some("1")).unwrap(), "1");
}

#[test]
fn a_unique_substring_matches_case_insensitively() {
    assert_eq!(select(&DEV_BOX, Some("rtk")).unwrap(), DEV_BOX[0]);
    assert_eq!(select(&DEV_BOX, Some("lg-tv")).unwrap(), DEV_BOX[1]);
    assert_eq!(select(&DEV_BOX, Some("QHD-hdr")).unwrap(), DEV_BOX[0]);
}

#[test]
fn a_value_matching_nothing_lists_every_monitor() {
    let error = select(&DEV_BOX, Some("nope")).unwrap_err();

    assert_eq!(
        error.to_string(),
        "no monitor matches 'nope'; choose one with --monitor <id|index>:\n  \
         1  RTK-RTK-QHD-HDR-01010101\n  \
         2  GSM-LG-TV-SSCR2-01010101\n\
         e.g. --monitor 1 or --monitor RTK-RTK-QHD-HDR-01010101"
    );
}

#[test]
fn an_ambiguous_substring_lists_only_the_candidates_with_their_global_index() {
    let keys = ["DELL-A", "RTK-QHD-1", "LG-TV", "RTK-QHD-2"];

    let error = select(&keys, Some("rtk-qhd")).unwrap_err();

    assert_eq!(
        error,
        SelectionError::Ambiguous {
            wanted: "rtk-qhd".to_owned(),
            candidates: vec![(2, info("RTK-QHD-1")), (4, info("RTK-QHD-2"))],
        }
    );
    assert_eq!(
        error.to_string(),
        "'rtk-qhd' matches 2 monitors; choose one with --monitor <id|index>:\n  \
         2  RTK-QHD-1\n  \
         4  RTK-QHD-2\n\
         e.g. --monitor 2 or --monitor RTK-QHD-1"
    );
}

#[test]
fn an_empty_value_matches_every_id_and_is_ambiguous() {
    let error = select(&DEV_BOX, Some("")).unwrap_err();

    assert!(
        matches!(error, SelectionError::Ambiguous { .. }),
        "{error:?}"
    );
    assert_eq!(select(&["ONLY"], Some("")).unwrap(), "ONLY");
}

#[test]
fn monitor_details_appear_in_the_listing() {
    let mut rtk = info(DEV_BOX[0]);
    rtk.manufacturer = Some("RTK".to_owned());
    rtk.model = Some("RTK QHD HDR".to_owned());

    let error = select_monitor(&[rtk, info(DEV_BOX[1])], None).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("\n  1  RTK-RTK-QHD-HDR-01010101  (RTK RTK QHD HDR)\n"),
        "{error}"
    );
}
