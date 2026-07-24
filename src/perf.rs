// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! P0-LAB-01 — feature-gated stage counters and coarse timers.
//!
//! When the `perf-profile` Cargo feature is **off** (default), every public
//! API here is a pure no-op with zero cost. When **on**, integer counters and
//! stage timers record a machine-readable JSON snapshot of one comparison.
//!
//! The profiled build is diagnostic only — never use it for final wall-time
//! acceptance numbers (OPERATING PLAN #4 / LCS_PERF_PLAN.md).

/// Coarse pipeline stages recorded by [`record_stage_ns`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Stage {
    /// Public API item.
    PackageOpen = 0,
    /// Public API item.
    Preprocess = 1,
    /// Public API item.
    Atomize = 2,
    /// Public API item.
    Unitize = 3,
    /// Public API item.
    Lcs = 4,
    /// Public API item.
    Produce = 5,
    /// Public API item.
    Serialize = 6,
    /// Public API item.
    Zip = 7,
}

const STAGE_COUNT: usize = 8;

/// Reset all counters and stage timers to zero. Safe to call mid-process for
/// isolation between trials when the feature is on.
pub fn reset() {
    #[cfg(feature = "perf-profile")]
    active::reset();
}

/// Add `nanos` to the cumulative time for `stage`.
#[inline]
pub fn record_stage_ns(stage: Stage, nanos: u64) {
    #[cfg(feature = "perf-profile")]
    active::record_stage_ns(stage, nanos);
    #[cfg(not(feature = "perf-profile"))]
    {
        let _ = (stage, nanos);
    }
}

/// Run `f` and, when profiling, accumulate its wall duration into `stage`.
#[inline]
pub fn time_stage<R>(stage: Stage, f: impl FnOnce() -> R) -> R {
    #[cfg(feature = "perf-profile")]
    {
        let t0 = std::time::Instant::now();
        let out = f();
        active::record_stage_ns(stage, t0.elapsed().as_nanos() as u64);
        out
    }
    #[cfg(not(feature = "perf-profile"))]
    {
        let _ = stage;
        f()
    }
}
/// Increment the LCS-call counter (no-op without `perf-profile`).
#[inline]
pub fn inc_lcs_calls() {
    #[cfg(feature = "perf-profile")]
    active::inc_lcs_calls();
}

/// Accumulate LCS window area `n × m` (no-op without `perf-profile`).
#[inline]
pub fn add_lcs_window_area(n: u64, m: u64) {
    #[cfg(feature = "perf-profile")]
    active::add_lcs_window_area(n, m);
    #[cfg(not(feature = "perf-profile"))]
    {
        let _ = (n, m);
    }
}

/// Increment correlation-run scan counter (no-op without `perf-profile`).
#[inline]
pub fn inc_corr_run_scans() {
    #[cfg(feature = "perf-profile")]
    active::inc_corr_run_scans();
}

/// Increment correlation-run hit counter (no-op without `perf-profile`).
#[inline]
pub fn inc_corr_run_hits() {
    #[cfg(feature = "perf-profile")]
    active::inc_corr_run_hits();
}

/// Add `n` unit clones to the clone counter (no-op without `perf-profile`).
#[inline]
pub fn add_unit_clones(n: u64) {
    #[cfg(feature = "perf-profile")]
    active::add_unit_clones(n);
    #[cfg(not(feature = "perf-profile"))]
    {
        let _ = n;
    }
}

/// Record the atom count for this compare (no-op without `perf-profile`).
#[inline]
pub fn set_atom_count(n: u64) {
    #[cfg(feature = "perf-profile")]
    active::set_atom_count(n);
    #[cfg(not(feature = "perf-profile"))]
    {
        let _ = n;
    }
}

/// Record the unit count for this compare (no-op without `perf-profile`).
#[inline]
pub fn set_unit_count(n: u64) {
    #[cfg(feature = "perf-profile")]
    active::set_unit_count(n);
    #[cfg(not(feature = "perf-profile"))]
    {
        let _ = n;
    }
}

/// Snapshot of counters/timers. Always available so tests can assert the
/// no-op path returns zeros when the feature is off.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    /// `stage_ns`.
    pub stage_ns: [u64; STAGE_COUNT],
    /// `lcs_calls`.
    pub lcs_calls: u64,
    /// `lcs_window_area`.
    pub lcs_window_area: u64,
    /// `corr_run_scans`.
    pub corr_run_scans: u64,
    /// `corr_run_hits`.
    pub corr_run_hits: u64,
    /// `unit_clones`.
    pub unit_clones: u64,
    /// `atoms`.
    pub atoms: u64,
    /// `units`.
    pub units: u64,
}

impl Snapshot {
    /// Emit a single-line JSON object (no trailing newline dependency).
    pub fn to_json(&self) -> String {
        format!(
            concat!(
                "{{\"stage_ns\":{{\"package_open\":{},\"preprocess\":{},",
                "\"atomize\":{},\"unitize\":{},\"lcs\":{},\"produce\":{},",
                "\"serialize\":{},\"zip\":{}}},",
                "\"lcs_calls\":{},\"lcs_window_area\":{},",
                "\"corr_run_scans\":{},\"corr_run_hits\":{},",
                "\"unit_clones\":{},\"atoms\":{},\"units\":{}}}"
            ),
            self.stage_ns[0],
            self.stage_ns[1],
            self.stage_ns[2],
            self.stage_ns[3],
            self.stage_ns[4],
            self.stage_ns[5],
            self.stage_ns[6],
            self.stage_ns[7],
            self.lcs_calls,
            self.lcs_window_area,
            self.corr_run_scans,
            self.corr_run_hits,
            self.unit_clones,
            self.atoms,
            self.units,
        )
    }
}

/// Capture the current counters without resetting.
pub fn snapshot() -> Snapshot {
    #[cfg(feature = "perf-profile")]
    {
        active::snapshot()
    }
    #[cfg(not(feature = "perf-profile"))]
    {
        Snapshot::default()
    }
}

/// Whether this build has the `perf-profile` feature compiled in.
pub const ENABLED: bool = cfg!(feature = "perf-profile");

#[cfg(feature = "perf-profile")]
mod active {
    use super::{STAGE_COUNT, Snapshot, Stage};
    use std::sync::atomic::{AtomicU64, Ordering};

    static STAGE_NS: [AtomicU64; STAGE_COUNT] = [
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
    ];
    static COUNTER_LCS_CALLS: AtomicU64 = AtomicU64::new(0);
    static COUNTER_LCS_WINDOW_AREA: AtomicU64 = AtomicU64::new(0);
    static COUNTER_CORR_RUN_SCANS: AtomicU64 = AtomicU64::new(0);
    static COUNTER_CORR_RUN_HITS: AtomicU64 = AtomicU64::new(0);
    static COUNTER_UNIT_CLONES: AtomicU64 = AtomicU64::new(0);
    static COUNTER_ATOMS: AtomicU64 = AtomicU64::new(0);
    static COUNTER_UNITS: AtomicU64 = AtomicU64::new(0);

    pub(super) fn reset() {
        for s in &STAGE_NS {
            s.store(0, Ordering::Relaxed);
        }
        COUNTER_LCS_CALLS.store(0, Ordering::Relaxed);
        COUNTER_LCS_WINDOW_AREA.store(0, Ordering::Relaxed);
        COUNTER_CORR_RUN_SCANS.store(0, Ordering::Relaxed);
        COUNTER_CORR_RUN_HITS.store(0, Ordering::Relaxed);
        COUNTER_UNIT_CLONES.store(0, Ordering::Relaxed);
        COUNTER_ATOMS.store(0, Ordering::Relaxed);
        COUNTER_UNITS.store(0, Ordering::Relaxed);
    }

    pub(super) fn record_stage_ns(stage: Stage, nanos: u64) {
        STAGE_NS[stage as usize].fetch_add(nanos, Ordering::Relaxed);
    }

    pub(super) fn inc_lcs_calls() {
        COUNTER_LCS_CALLS.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn add_lcs_window_area(n: u64, m: u64) {
        COUNTER_LCS_WINDOW_AREA.fetch_add(n.saturating_mul(m), Ordering::Relaxed);
    }

    pub(super) fn inc_corr_run_scans() {
        COUNTER_CORR_RUN_SCANS.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn inc_corr_run_hits() {
        COUNTER_CORR_RUN_HITS.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn add_unit_clones(n: u64) {
        COUNTER_UNIT_CLONES.fetch_add(n, Ordering::Relaxed);
    }

    pub(super) fn set_atom_count(n: u64) {
        COUNTER_ATOMS.store(n, Ordering::Relaxed);
    }

    pub(super) fn set_unit_count(n: u64) {
        COUNTER_UNITS.store(n, Ordering::Relaxed);
    }

    pub(super) fn snapshot() -> Snapshot {
        Snapshot {
            stage_ns: [
                STAGE_NS[0].load(Ordering::Relaxed),
                STAGE_NS[1].load(Ordering::Relaxed),
                STAGE_NS[2].load(Ordering::Relaxed),
                STAGE_NS[3].load(Ordering::Relaxed),
                STAGE_NS[4].load(Ordering::Relaxed),
                STAGE_NS[5].load(Ordering::Relaxed),
                STAGE_NS[6].load(Ordering::Relaxed),
                STAGE_NS[7].load(Ordering::Relaxed),
            ],
            lcs_calls: COUNTER_LCS_CALLS.load(Ordering::Relaxed),
            lcs_window_area: COUNTER_LCS_WINDOW_AREA.load(Ordering::Relaxed),
            corr_run_scans: COUNTER_CORR_RUN_SCANS.load(Ordering::Relaxed),
            corr_run_hits: COUNTER_CORR_RUN_HITS.load(Ordering::Relaxed),
            unit_clones: COUNTER_UNIT_CLONES.load(Ordering::Relaxed),
            atoms: COUNTER_ATOMS.load(Ordering::Relaxed),
            units: COUNTER_UNITS.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_json_is_valid_shape() {
        let s = Snapshot {
            stage_ns: [1, 2, 3, 4, 5, 6, 7, 8],
            lcs_calls: 9,
            lcs_window_area: 10,
            corr_run_scans: 11,
            corr_run_hits: 12,
            unit_clones: 13,
            atoms: 14,
            units: 15,
        };
        let j = s.to_json();
        assert!(j.starts_with('{') && j.ends_with('}'));
        assert!(j.contains("\"lcs_calls\":9"));
        assert!(j.contains("\"package_open\":1"));
        assert!(j.contains("\"units\":15"));
    }

    #[test]
    fn reset_and_record_are_safe_without_feature() {
        reset();
        record_stage_ns(Stage::Lcs, 42);
        inc_lcs_calls();
        add_lcs_window_area(3, 4);
        inc_corr_run_scans();
        inc_corr_run_hits();
        add_unit_clones(2);
        set_atom_count(7);
        set_unit_count(8);
        let s = snapshot();
        if ENABLED {
            assert_eq!(s.stage_ns[Stage::Lcs as usize], 42);
            assert_eq!(s.lcs_calls, 1);
            assert_eq!(s.lcs_window_area, 12);
            assert_eq!(s.corr_run_scans, 1);
            assert_eq!(s.corr_run_hits, 1);
            assert_eq!(s.unit_clones, 2);
            assert_eq!(s.atoms, 7);
            assert_eq!(s.units, 8);
        } else {
            assert_eq!(s, Snapshot::default());
        }
        reset();
    }

    #[test]
    fn time_stage_returns_closure_result() {
        let v = time_stage(Stage::Atomize, || 99u32);
        assert_eq!(v, 99);
    }
}
