//! `CachingMonitorBackend` over the in-memory fake, with a real temporary
//! cache directory — each test gets its own and removes it when done.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use ddc_adapters::{BackendCall, CachingMonitorBackend, FakeMonitor, InMemoryMonitorBackend};
use ddc_core::app::SoftwareOsd;
use ddc_core::domain::{Confirm, DdcError, MonitorId, MonitorInfo, VcpCode};
use ddc_core::ports::{MonitorBackend, MonitorControl};

const CAPS: &str = "(prot(monitor)type(LCD)model(FAKE)vcp(10 12 60(0F 11)))";
const NEW_CAPS: &str = "(prot(monitor)type(LCD)model(FAKE)vcp(10 12 14(01 02)))";
const OLD_CAPS: &str = "(prot(monitor)type(LCD)model(OLD)vcp(10))";
const KEY: &str = "FAKE-CACHE-01";

/// A fresh, empty directory under the system temp dir, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> io::Result<Self> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ddc-adapters-caching-{}-{label}-{serial}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }

    /// Paths of the entries in the directory, sorted.
    fn files(&self) -> io::Result<Vec<PathBuf>> {
        let mut files = fs::read_dir(&self.0)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<io::Result<Vec<_>>>()?;
        files.sort();
        Ok(files)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn id() -> MonitorId {
    MonitorId::new(KEY)
}

fn cache_file(dir: &TempDir) -> PathBuf {
    dir.path().join(format!("{KEY}.caps"))
}

fn monitor(caps: &str) -> FakeMonitor {
    FakeMonitor::new(MonitorInfo {
        id: id(),
        manufacturer: Some("FAK".to_owned()),
        model: Some("CACHE".to_owned()),
        serial: None,
    })
    .with_capabilities(caps)
    .with_value(VcpCode::BRIGHTNESS, 50, 100)
}

fn backend(monitor: FakeMonitor) -> InMemoryMonitorBackend {
    InMemoryMonitorBackend::builder().monitor(monitor).build()
}

fn capabilities_reads(fake: &InMemoryMonitorBackend) -> usize {
    fake.calls()
        .iter()
        .filter(|call| matches!(call, BackendCall::ReadCapabilities(_)))
        .count()
}

#[test]
fn caching_backend_reads_capabilities_from_disk_without_calling_wrapped_backend_after_first_fetch()
{
    let dir = TempDir::new("disk-hit").unwrap();
    let first_fake = backend(monitor(CAPS));
    let first = CachingMonitorBackend::new(first_fake.clone(), dir.path().to_path_buf());

    assert_eq!(first.read_capabilities(&id()).unwrap(), CAPS);
    assert_eq!(capabilities_reads(&first_fake), 1);
    drop(first);

    let second_fake = backend(monitor(NEW_CAPS));
    let second = CachingMonitorBackend::new(second_fake.clone(), dir.path().to_path_buf());

    assert_eq!(second.read_capabilities(&id()).unwrap(), CAPS);
    assert_eq!(second.read_capabilities(&id()).unwrap(), CAPS);
    assert_eq!(capabilities_reads(&second_fake), 0);
    assert!(second_fake.calls().is_empty());
}

#[test]
fn caching_backend_never_persists_failures_and_refresh_forces_a_fresh_overwrite() {
    let dir = TempDir::new("refresh").unwrap();
    let flaky = backend(monitor(CAPS).with_transient_capabilities_failures(1));
    let caching = CachingMonitorBackend::new(flaky.clone(), dir.path().to_path_buf());

    let failed = caching.read_capabilities(&id());
    assert!(matches!(failed, Err(DdcError::Transport(_))));
    assert!(!cache_file(&dir).exists());

    assert_eq!(caching.read_capabilities(&id()).unwrap(), CAPS);
    assert_eq!(fs::read_to_string(cache_file(&dir)).unwrap(), CAPS);
    assert_eq!(capabilities_reads(&flaky), 2);

    fs::write(cache_file(&dir), OLD_CAPS).unwrap();
    let fresh = backend(monitor(NEW_CAPS));
    let stale = CachingMonitorBackend::new(fresh.clone(), dir.path().to_path_buf());
    assert_eq!(stale.read_capabilities(&id()).unwrap(), OLD_CAPS);
    assert_eq!(capabilities_reads(&fresh), 0);

    stale.invalidate(&id());

    assert_eq!(stale.read_capabilities(&id()).unwrap(), NEW_CAPS);
    assert_eq!(capabilities_reads(&fresh), 1);
    assert_eq!(fs::read_to_string(cache_file(&dir)).unwrap(), NEW_CAPS);
    assert_eq!(stale.read_capabilities(&id()).unwrap(), NEW_CAPS);
    assert_eq!(capabilities_reads(&fresh), 1);
}

#[test]
fn enumerate_is_never_cached() {
    let dir = TempDir::new("enumerate").unwrap();
    let fake = backend(monitor(CAPS));
    let caching = CachingMonitorBackend::new(fake.clone(), dir.path().to_path_buf());

    assert_eq!(caching.enumerate().unwrap().len(), 1);
    assert_eq!(caching.enumerate().unwrap().len(), 1);

    assert_eq!(
        fake.calls(),
        [BackendCall::Enumerate, BackendCall::Enumerate]
    );
    assert!(dir.files().unwrap().is_empty());
}

#[test]
fn software_osd_runs_on_a_borrowed_cache_that_can_still_be_invalidated() {
    let dir = TempDir::new("borrowed").unwrap();
    let fake = backend(monitor(CAPS));
    let caching = CachingMonitorBackend::new(fake.clone(), dir.path().to_path_buf());
    let osd = SoftwareOsd::new(&caching);

    assert_eq!(osd.list_monitors().unwrap().len(), 1);
    let reading = osd.get_feature(&id(), VcpCode::BRIGHTNESS).unwrap();
    assert!(reading.declared_in_capabilities);
    let written = osd
        .set_feature(&id(), VcpCode::BRIGHTNESS, 60, Confirm::No)
        .unwrap();
    assert_eq!(written.current, 60);
    assert_eq!(fs::read_to_string(cache_file(&dir)).unwrap(), CAPS);

    caching.invalidate(&id());
    assert!(!cache_file(&dir).exists());

    assert_eq!(
        fake.calls(),
        [
            BackendCall::Enumerate,
            BackendCall::ReadCapabilities(id()),
            BackendCall::ReadVcp(id(), VcpCode::BRIGHTNESS),
            BackendCall::WriteVcp(id(), VcpCode::BRIGHTNESS, 60),
            BackendCall::ReadVcp(id(), VcpCode::BRIGHTNESS),
        ]
    );
}

#[test]
fn owned_cache_delegates_vcp_reads_and_writes() {
    let dir = TempDir::new("owned").unwrap();
    let fake = backend(monitor(CAPS));
    let caching = CachingMonitorBackend::new(fake.clone(), dir.path().to_path_buf());

    caching.write_vcp(&id(), VcpCode::BRIGHTNESS, 70).unwrap();
    let value = caching.read_vcp(&id(), VcpCode::BRIGHTNESS).unwrap();

    assert_eq!(value.current, 70);
    assert_eq!(
        fake.calls(),
        [
            BackendCall::WriteVcp(id(), VcpCode::BRIGHTNESS, 70),
            BackendCall::ReadVcp(id(), VcpCode::BRIGHTNESS),
        ]
    );
}

#[test]
fn an_unwritable_cache_dir_never_fails_a_read() {
    let dir = TempDir::new("unwritable").unwrap();
    let not_a_dir = dir.path().join("occupied");
    fs::write(&not_a_dir, "a file, not a directory").unwrap();
    let fake = backend(monitor(CAPS));
    let caching = CachingMonitorBackend::new(fake.clone(), not_a_dir.join("caps"));

    assert_eq!(caching.read_capabilities(&id()).unwrap(), CAPS);
    assert_eq!(caching.read_capabilities(&id()).unwrap(), CAPS);

    assert_eq!(capabilities_reads(&fake), 2);
    assert_eq!(dir.files().unwrap(), [not_a_dir]);
}

#[test]
fn an_empty_cache_file_is_a_miss_and_gets_overwritten() {
    let dir = TempDir::new("empty").unwrap();
    fs::write(cache_file(&dir), "").unwrap();
    let fake = backend(monitor(CAPS));
    let caching = CachingMonitorBackend::new(fake.clone(), dir.path().to_path_buf());

    assert_eq!(caching.read_capabilities(&id()).unwrap(), CAPS);

    assert_eq!(capabilities_reads(&fake), 1);
    assert_eq!(fs::read_to_string(cache_file(&dir)).unwrap(), CAPS);
}

#[test]
fn invalidating_an_uncached_monitor_just_reads_it_next_time() {
    let dir = TempDir::new("uncached").unwrap();
    let fake = backend(monitor(CAPS));
    let caching = CachingMonitorBackend::new(fake.clone(), dir.path().to_path_buf());

    caching.invalidate(&id());

    assert_eq!(caching.read_capabilities(&id()).unwrap(), CAPS);
    assert_eq!(capabilities_reads(&fake), 1);
}

#[test]
fn a_cache_file_that_cannot_be_replaced_is_bypassed_on_every_read() {
    let dir = TempDir::new("blocked").unwrap();
    let file = cache_file(&dir);
    fs::create_dir(&file).unwrap();
    let fake = backend(monitor(CAPS));
    let caching = CachingMonitorBackend::new(fake.clone(), dir.path().to_path_buf());

    caching.invalidate(&id());
    assert!(file.is_dir());

    assert_eq!(caching.read_capabilities(&id()).unwrap(), CAPS);
    assert_eq!(caching.read_capabilities(&id()).unwrap(), CAPS);
    assert_eq!(capabilities_reads(&fake), 2);
    assert_eq!(dir.files().unwrap(), [file]);
}

/// Removal fails in a read-only directory while the file stays readable:
/// the invalidation alone must keep the stale file from being served.
#[cfg(unix)]
#[test]
fn invalidation_holds_when_the_stale_file_cannot_be_removed() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new("read-only").unwrap();
    fs::write(cache_file(&dir), OLD_CAPS).unwrap();
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let fake = backend(monitor(CAPS));
    let caching = CachingMonitorBackend::new(fake.clone(), dir.path().to_path_buf());

    caching.invalidate(&id());
    let removed = !cache_file(&dir).exists();
    let first = caching.read_capabilities(&id());
    let second = caching.read_capabilities(&id());
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();

    if removed {
        eprintln!("skipped: running with permission to write a read-only directory");
        return;
    }
    assert_eq!(first.unwrap(), CAPS);
    assert_eq!(second.unwrap(), CAPS);
    assert_eq!(capabilities_reads(&fake), 2);
    assert_eq!(fs::read_to_string(cache_file(&dir)).unwrap(), OLD_CAPS);
}

#[test]
fn ids_with_path_characters_stay_inside_the_cache_dir() {
    let dir = TempDir::new("traversal").unwrap();
    let cache = dir.path().join("caps");
    let sneaky = MonitorId::new("../escape");
    let fake = InMemoryMonitorBackend::builder()
        .monitor(
            FakeMonitor::new(MonitorInfo {
                id: sneaky.clone(),
                manufacturer: None,
                model: None,
                serial: None,
            })
            .with_capabilities(CAPS),
        )
        .build();
    let caching = CachingMonitorBackend::new(fake, cache.clone());

    assert_eq!(caching.read_capabilities(&sneaky).unwrap(), CAPS);

    assert_eq!(dir.files().unwrap(), [cache.as_path()]);
    assert_eq!(
        fs::read_to_string(cache.join("_2E_2E_2Fescape.caps")).unwrap(),
        CAPS
    );
}
