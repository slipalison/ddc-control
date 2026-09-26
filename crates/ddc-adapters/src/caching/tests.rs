use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use ddc_core::domain::MonitorId;

use super::{Platform, cache_dir_for, default_cache_dir, file_stem};

fn stem(key: &str) -> String {
    file_stem(&MonitorId::new(key))
}

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let vars: HashMap<String, OsString> = pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), OsString::from(value)))
        .collect();
    move |name| vars.get(name).cloned()
}

fn caps_under(base: &Path) -> PathBuf {
    base.join("ddc-control").join("caps")
}

#[test]
fn file_stem_keeps_ascii_letters_digits_and_dashes() {
    assert_eq!(stem("RTK-RTK-QHD-HDR-01010101"), "RTK-RTK-QHD-HDR-01010101");
}

#[test]
fn file_stem_hex_encodes_every_other_byte() {
    assert_eq!(stem("RTK-01010101#2"), "RTK-01010101_232");
    assert_eq!(stem("a_b"), "a_5Fb");
    assert_eq!(stem("é"), "_C3_A9");
}

#[test]
fn file_stem_never_contains_separators_or_dots() {
    for key in ["../../etc/passwd", "..", "a/b", r"C:\x\y", "a.caps"] {
        let encoded = stem(key);
        assert!(
            !encoded.contains(['/', '\\', '.', ':']),
            "{key:?} -> {encoded:?}"
        );
    }
    assert_eq!(stem(".."), "_2E_2E");
    assert_eq!(stem("a/b"), "a_2Fb");
}

#[test]
fn file_stem_is_injective_across_lookalike_ids() {
    let keys = ["a_2F", "a/", "a_", "a", "a#2", "a_232", "a-2", "A"];
    let stems: std::collections::HashSet<String> = keys.iter().map(|key| stem(key)).collect();
    assert_eq!(stems.len(), keys.len());
}

#[test]
fn unix_prefers_an_absolute_xdg_cache_home() {
    let dir = cache_dir_for(
        Platform::Unix,
        env(&[("XDG_CACHE_HOME", "/xdg"), ("HOME", "/home/u")]),
    );

    assert_eq!(dir, Some(caps_under(Path::new("/xdg"))));
}

#[test]
fn unix_ignores_a_relative_or_empty_xdg_cache_home() {
    for xdg in ["relative/cache", ""] {
        let dir = cache_dir_for(
            Platform::Unix,
            env(&[("XDG_CACHE_HOME", xdg), ("HOME", "/home/u")]),
        );

        assert_eq!(
            dir,
            Some(caps_under(&Path::new("/home/u").join(".cache"))),
            "XDG_CACHE_HOME={xdg:?}"
        );
    }
}

#[test]
fn unix_falls_back_to_home_dot_cache() {
    let dir = cache_dir_for(Platform::Unix, env(&[("HOME", "/home/u")]));

    assert_eq!(dir, Some(caps_under(&Path::new("/home/u").join(".cache"))));
}

#[test]
fn unix_without_a_usable_directory_has_no_cache_dir() {
    assert_eq!(cache_dir_for(Platform::Unix, env(&[])), None);
    assert_eq!(
        cache_dir_for(Platform::Unix, env(&[("HOME", "relative")])),
        None
    );
    assert_eq!(cache_dir_for(Platform::Unix, env(&[("HOME", "")])), None);
}

#[test]
fn windows_uses_local_app_data_and_ignores_unix_variables() {
    let local = r"C:\Users\u\AppData\Local";
    let dir = cache_dir_for(
        Platform::Windows,
        env(&[
            ("LOCALAPPDATA", local),
            ("XDG_CACHE_HOME", "/xdg"),
            ("HOME", "/home/u"),
        ]),
    );

    assert_eq!(dir, Some(caps_under(Path::new(local))));
}

#[test]
fn windows_without_local_app_data_has_no_cache_dir() {
    assert_eq!(
        cache_dir_for(Platform::Windows, env(&[("HOME", "/home/u")])),
        None
    );
    assert_eq!(
        cache_dir_for(Platform::Windows, env(&[("LOCALAPPDATA", "")])),
        None
    );
}

#[test]
fn default_cache_dir_ends_in_the_project_caps_directory() {
    if let Some(dir) = default_cache_dir() {
        assert!(
            dir.ends_with(Path::new("ddc-control").join("caps")),
            "{dir:?}"
        );
    }
}
