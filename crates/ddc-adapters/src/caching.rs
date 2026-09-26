//! A `MonitorBackend` decorator that keeps capabilities strings on disk.
//!
//! Reading a capabilities string costs seconds on real monitors, and a CLI
//! invocation lives for one command, so the in-process cache of the core
//! does not help it. This decorator stores each successful read in one file
//! per monitor and serves it from there on later runs.

use std::collections::HashSet;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use ddc_core::domain::{DdcError, MonitorId, MonitorInfo, VcpCode, VcpValue};
use ddc_core::ports::MonitorBackend;

/// Extension of a cached capabilities file.
const CACHE_EXTENSION: &str = "caps";

/// Tells apart the temporary files of concurrent writes from one process.
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Wraps a [`MonitorBackend`] and persists `read_capabilities` per
/// [`MonitorId`] under `cache_dir`.
///
/// Only successful reads are stored; failures always reach the wrapped
/// backend again. Every other call is delegated untouched — enumeration in
/// particular is never cached. Cache I/O never fails an operation: an
/// unreadable or empty file is a miss, and a failed write is ignored.
#[derive(Debug)]
pub struct CachingMonitorBackend<B> {
    inner: B,
    cache_dir: PathBuf,
    invalidated: Mutex<HashSet<MonitorId>>,
}

impl<B: MonitorBackend> CachingMonitorBackend<B> {
    /// Wraps `inner`, keeping capabilities files in `cache_dir`, which is
    /// created on the first write.
    pub fn new(inner: B, cache_dir: PathBuf) -> Self {
        Self {
            inner,
            cache_dir,
            invalidated: Mutex::new(HashSet::new()),
        }
    }

    /// Forgets the cached capabilities of `id`: the next read asks the
    /// wrapped backend and overwrites the file. Holds even when the file
    /// cannot be removed.
    pub fn invalidate(&self, id: &MonitorId) {
        lock(&self.invalidated).insert(id.clone());
        let _ = fs::remove_file(self.cache_file(id));
    }

    fn cache_file(&self, id: &MonitorId) -> PathBuf {
        self.cache_dir
            .join(format!("{}.{CACHE_EXTENSION}", file_stem(id)))
    }

    fn cached(&self, id: &MonitorId) -> Option<String> {
        if lock(&self.invalidated).contains(id) {
            return None;
        }
        let raw = fs::read_to_string(self.cache_file(id)).ok()?;
        (!raw.is_empty()).then_some(raw)
    }

    fn store(&self, id: &MonitorId, raw: &str) {
        if write_atomically(&self.cache_dir, &self.cache_file(id), raw).is_ok() {
            lock(&self.invalidated).remove(id);
        }
    }
}

impl<B: MonitorBackend> MonitorBackend for CachingMonitorBackend<B> {
    fn enumerate(&self) -> Result<Vec<MonitorInfo>, DdcError> {
        self.inner.enumerate()
    }

    fn read_capabilities(&self, id: &MonitorId) -> Result<String, DdcError> {
        if let Some(raw) = self.cached(id) {
            return Ok(raw);
        }
        let raw = self.inner.read_capabilities(id)?;
        self.store(id, &raw);
        Ok(raw)
    }

    fn read_vcp(&self, id: &MonitorId, code: VcpCode) -> Result<VcpValue, DdcError> {
        self.inner.read_vcp(id, code)
    }

    fn write_vcp(&self, id: &MonitorId, code: VcpCode, value: u16) -> Result<(), DdcError> {
        self.inner.write_vcp(id, code, value)
    }
}

/// Lets a composition root hand a borrowed cache to the core and still call
/// [`CachingMonitorBackend::invalidate`] on it.
impl<B: MonitorBackend> MonitorBackend for &CachingMonitorBackend<B> {
    fn enumerate(&self) -> Result<Vec<MonitorInfo>, DdcError> {
        (**self).enumerate()
    }

    fn read_capabilities(&self, id: &MonitorId) -> Result<String, DdcError> {
        (**self).read_capabilities(id)
    }

    fn read_vcp(&self, id: &MonitorId, code: VcpCode) -> Result<VcpValue, DdcError> {
        (**self).read_vcp(id, code)
    }

    fn write_vcp(&self, id: &MonitorId, code: VcpCode, value: u16) -> Result<(), DdcError> {
        (**self).write_vcp(id, code, value)
    }
}

/// The per-user directory for cached capabilities: `%LOCALAPPDATA%` on
/// Windows; elsewhere `$XDG_CACHE_HOME`, else `$HOME/.cache`; followed by
/// `ddc-control/caps`. `None` when the environment names no such directory.
pub fn default_cache_dir() -> Option<PathBuf> {
    let platform = if cfg!(windows) {
        Platform::Windows
    } else {
        Platform::Unix
    };
    cache_dir_for(platform, |name| std::env::var_os(name))
}

/// Which convention [`cache_dir_for`] follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    Windows,
    Unix,
}

/// [`default_cache_dir`] with the environment passed in, so both
/// conventions can be tested on any host.
fn cache_dir_for(platform: Platform, var: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let base = match platform {
        Platform::Windows => non_empty(var("LOCALAPPDATA"))?,
        // The XDG spec says a relative `XDG_CACHE_HOME` is invalid and must
        // be ignored; a relative `HOME` would scatter caches across cwds.
        Platform::Unix => rooted(var("XDG_CACHE_HOME"))
            .or_else(|| rooted(var("HOME")).map(|home| home.join(".cache")))?,
    };
    Some(base.join("ddc-control").join("caps"))
}

fn non_empty(value: Option<OsString>) -> Option<PathBuf> {
    value.filter(|value| !value.is_empty()).map(PathBuf::from)
}

fn rooted(value: Option<OsString>) -> Option<PathBuf> {
    non_empty(value).filter(|path| path.has_root())
}

/// File name stem for `id`: ASCII letters, digits and `-` are kept, every
/// other byte becomes `_XX` (uppercase hex). Injective, and free of path
/// separators and dots, so no id can name a file outside the cache dir.
fn file_stem(id: &MonitorId) -> String {
    let mut stem = String::with_capacity(id.as_str().len());
    for byte in id.as_str().bytes() {
        if byte.is_ascii_alphanumeric() || byte == b'-' {
            stem.push(char::from(byte));
        } else {
            let _ = write!(stem, "_{byte:02X}");
        }
    }
    stem
}

/// Writes `contents` to a unique temporary file in `dir`, then renames it
/// over `target`, so a reader never sees a half-written file.
fn write_atomically(dir: &Path, target: &Path, contents: &str) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let serial = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp = dir.join(format!(".{}-{serial}.tmp", std::process::id()));
    let written = fs::write(&temp, contents).and_then(|()| fs::rename(&temp, target));
    if written.is_err() {
        let _ = fs::remove_file(&temp);
    }
    written
}

/// The set of invalidated ids is plain data, valid after any panic, so a
/// poisoned lock is recovered rather than propagated.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests;
