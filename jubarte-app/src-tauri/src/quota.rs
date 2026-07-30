//! Free-tier quota: every install gets [`FREE_LIMIT`] redlines for free; after
//! that, `create_redline` requires an active subscription.
//!
//! The count is persisted in the app data dir (inside the App Sandbox
//! container). Like every client-side gate this is a deterrent, not DRM — the
//! authoritative entitlement record stays with Apple/the verify backend.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::Manager;

/// How many redlines an install may produce before the subscription gate.
pub const FREE_LIMIT: u32 = 5;

/// Marker error returned by `create_redline` when the free allowance is spent
/// and no subscription is active. The frontend matches on this exact string to
/// open the paywall instead of showing an error toast.
pub const FREE_LIMIT_ERR: &str = "FREE_LIMIT_REACHED";

/// Managed state: the persisted number of redlines produced so far.
pub struct Quota(pub Mutex<u32>);

#[derive(Serialize)]
pub struct QuotaStatus {
    pub used: u32,
    pub limit: u32,
    pub remaining: u32,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct Persisted {
    used: u32,
}

fn store_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("usage.json"))
}

/// Read the persisted use count; any unreadable/absent/corrupt file counts as 0
/// (never lock a user out because of a bad disk read).
pub fn read_used(path: &Path) -> u32 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Persisted>(&s).ok())
        .map(|p| p.used)
        .unwrap_or(0)
}

/// Persist the use count. Best-effort: a write failure only means the count
/// resets on relaunch, which errs in the user's favour.
pub fn write_used(path: &Path, used: u32) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string(&Persisted { used }) {
        let _ = std::fs::write(path, json);
    }
}

/// Build a [`QuotaStatus`] from a raw use count (pure; unit-tested).
pub fn status_from_used(used: u32) -> QuotaStatus {
    QuotaStatus {
        used,
        limit: FREE_LIMIT,
        remaining: FREE_LIMIT.saturating_sub(used),
    }
}

/// Atomically try to reserve one free-tier slot under `counter`.
///
/// Returns `true` and increments when `used < FREE_LIMIT`; otherwise `false`.
/// Used by [`try_reserve_free_use`] and by concurrent unit tests
/// (CodeRabbit #3623452481 / #3623452484).
pub fn try_reserve_counter(counter: &Mutex<u32>) -> bool {
    let mut used = counter.lock().unwrap();
    if *used >= FREE_LIMIT {
        return false;
    }
    *used = used.saturating_add(1);
    true
}

/// Roll back one free-tier reservation under `counter`.
pub fn release_counter(counter: &Mutex<u32>) {
    let mut used = counter.lock().unwrap();
    *used = used.saturating_sub(1);
}

/// Bump the free-tier counter by one (analytics path for entitled users).
pub fn bump_counter(counter: &Mutex<u32>) -> u32 {
    let mut used = counter.lock().unwrap();
    *used = used.saturating_add(1);
    *used
}

/// Load the persisted count into managed state. Call once from `setup`.
pub fn init(app: &tauri::AppHandle) {
    let used = store_path(app).map(|p| read_used(&p)).unwrap_or(0);
    app.manage(Quota(Mutex::new(used)));
}

/// Current use count from managed state.
pub fn used(app: &tauri::AppHandle) -> u32 {
    *app.state::<Quota>().0.lock().unwrap()
}

/// Atomically reserve one free-tier redline under the mutex + persist.
pub fn try_reserve_free_use(app: &tauri::AppHandle) -> bool {
    let quota = app.state::<Quota>();
    if !try_reserve_counter(&quota.0) {
        return false;
    }
    let n = *quota.0.lock().unwrap();
    if let Some(path) = store_path(app) {
        write_used(&path, n);
    }
    true
}

/// Roll back a reservation when the comparison fails after a successful reserve.
pub fn release_free_use(app: &tauri::AppHandle) {
    let quota = app.state::<Quota>();
    release_counter(&quota.0);
    let n = *quota.0.lock().unwrap();
    if let Some(path) = store_path(app) {
        write_used(&path, n);
    }
}

/// Record one successful redline for the entitled (paid) path.
pub fn record_use(app: &tauri::AppHandle) {
    let quota = app.state::<Quota>();
    let n = bump_counter(&quota.0);
    if let Some(path) = store_path(app) {
        write_used(&path, n);
    }
}

/// Free-tier status for the frontend (badge + paywall trigger).
#[tauri::command]
pub fn quota_status(app: tauri::AppHandle) -> QuotaStatus {
    status_from_used(used(&app))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::thread;

    fn tmp_file(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("jubarte-quota-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("usage.json")
    }

    #[test]
    fn missing_file_reads_as_zero() {
        assert_eq!(read_used(Path::new("/nonexistent/usage.json")), 0);
    }

    #[test]
    fn round_trips_the_count() {
        let path = tmp_file("roundtrip");
        write_used(&path, 3);
        assert_eq!(read_used(&path), 3);
        write_used(&path, 5);
        assert_eq!(read_used(&path), 5);
    }

    #[test]
    fn corrupt_file_reads_as_zero() {
        let path = tmp_file("corrupt");
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(&path, "{not json").unwrap();
        assert_eq!(read_used(&path), 0);
    }

    #[test]
    fn empty_file_reads_as_zero() {
        let path = tmp_file("empty");
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(&path, "").unwrap();
        assert_eq!(read_used(&path), 0);
    }

    #[test]
    fn write_creates_parent_dirs() {
        let path = tmp_file("nested/deep/usage.json");
        write_used(&path, 2);
        assert_eq!(read_used(&path), 2);
    }

    #[test]
    fn status_from_used_math() {
        let s = status_from_used(0);
        assert_eq!(s.used, 0);
        assert_eq!(s.limit, FREE_LIMIT);
        assert_eq!(s.remaining, FREE_LIMIT);

        let s = status_from_used(FREE_LIMIT);
        assert_eq!(s.remaining, 0);

        let s = status_from_used(FREE_LIMIT + 3);
        assert_eq!(s.remaining, 0);
        assert_eq!(s.used, FREE_LIMIT + 3);
    }

    #[test]
    fn limit_math_never_underflows() {
        assert_eq!(FREE_LIMIT.saturating_sub(FREE_LIMIT + 10), 0);
    }

    #[test]
    fn free_limit_err_marker_is_stable() {
        // Frontend matches this exact string.
        assert_eq!(FREE_LIMIT_ERR, "FREE_LIMIT_REACHED");
    }

    #[test]
    fn try_reserve_never_exceeds_free_limit() {
        let counter = Mutex::new(0u32);
        let mut ok = 0u32;
        for _ in 0..(FREE_LIMIT + 10) {
            if try_reserve_counter(&counter) {
                ok += 1;
            }
        }
        assert_eq!(ok, FREE_LIMIT);
        assert_eq!(*counter.lock().unwrap(), FREE_LIMIT);
        assert!(!try_reserve_counter(&counter));
    }

    #[test]
    fn release_after_failed_compare_restores_slot() {
        let counter = Mutex::new(FREE_LIMIT - 1);
        assert!(try_reserve_counter(&counter));
        assert_eq!(*counter.lock().unwrap(), FREE_LIMIT);
        assert!(!try_reserve_counter(&counter));
        release_counter(&counter);
        assert_eq!(*counter.lock().unwrap(), FREE_LIMIT - 1);
        assert!(try_reserve_counter(&counter));
    }

    #[test]
    fn release_at_zero_saturates() {
        let counter = Mutex::new(0u32);
        release_counter(&counter);
        assert_eq!(*counter.lock().unwrap(), 0);
    }

    #[test]
    fn bump_counter_increments() {
        let counter = Mutex::new(2u32);
        assert_eq!(bump_counter(&counter), 3);
        assert_eq!(bump_counter(&counter), 4);
    }

    #[test]
    fn concurrent_last_slot_admits_exactly_one() {
        let counter = Arc::new(Mutex::new(FREE_LIMIT - 1));
        let wins = Arc::new(AtomicU32::new(0));
        let mut handles = Vec::new();
        for _ in 0..16 {
            let c = Arc::clone(&counter);
            let w = Arc::clone(&wins);
            handles.push(thread::spawn(move || {
                if try_reserve_counter(&c) {
                    w.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(wins.load(Ordering::SeqCst), 1);
        assert_eq!(*counter.lock().unwrap(), FREE_LIMIT);
    }

    #[test]
    fn persisted_json_roundtrip() {
        let p = Persisted { used: 4 };
        let j = serde_json::to_string(&p).unwrap();
        assert!(j.contains("\"used\""));
        let q: Persisted = serde_json::from_str(&j).unwrap();
        assert_eq!(p, q);
    }

    #[test]
    fn concurrent_from_zero_admits_exactly_free_limit() {
        let counter = Arc::new(Mutex::new(0u32));
        let wins = Arc::new(AtomicU32::new(0));
        let mut handles = Vec::new();
        for _ in 0..32 {
            let c = Arc::clone(&counter);
            let w = Arc::clone(&wins);
            handles.push(thread::spawn(move || {
                if try_reserve_counter(&c) {
                    w.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(wins.load(Ordering::SeqCst), FREE_LIMIT);
        assert_eq!(*counter.lock().unwrap(), FREE_LIMIT);
    }
}
