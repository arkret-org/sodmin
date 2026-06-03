//! DTO shapes for the E2EE covered_frontier lag admin surface (Stream H',
//! H'7).
//!
//! Read from `GET /api/admin/v1/spaces/{id}/mls/covered-frontier` —
//! soland projects the lattice or-set state for the
//! `ck:cell:cx.component.mls.covered_frontier.v1:<realm_id>` cell. The
//! cell records which governance Anchors / Moves the MLS group has
//! acknowledged. We compare the current governance frontier against the
//! covered set to compute a *lag count* — how many governance Moves the
//! MLS group has yet to acknowledge.

use serde::{Deserialize, Serialize};

/// Default warning threshold (in Moves). Above this, the page paints the
/// lag in red so the admin sees the urgency. Used by the page render
/// path; pure so it can be changed without re-pulling everything.
pub const DEFAULT_LAG_WARN_THRESHOLD: u64 = 5;

/// Snapshot returned by the covered-frontier admin describe endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoveredFrontierSnapshot {
    pub realm_id: String,
    /// Current MLS group epoch — a monotonically-increasing integer that
    /// the group bumps on every Add/Remove/Update.
    #[serde(default)]
    pub mls_epoch: u64,
    /// Move-ids the governance frontier currently includes. Order is the
    /// canonical frontier order soland chose.
    #[serde(default)]
    pub governance_frontier: Vec<String>,
    /// Move-ids the MLS group has *acknowledged* (folded into the
    /// covered_frontier or-set value). A subset of `governance_frontier`
    /// in the steady state; lag = |governance_frontier - covered_frontier|.
    #[serde(default)]
    pub covered_frontier: Vec<String>,
    /// Latest Anchor id whose frontier is reflected in `governance_frontier`.
    #[serde(default)]
    pub latest_anchor_id: Option<String>,
    #[serde(default)]
    pub last_covered_at: Option<String>,
}

/// Response from the `POST /api/admin/v1/spaces/{id}/mls/covered-frontier/advance`
/// route. soland reports the new lag count after the override Move
/// landed; the page uses this to render an immediate "now caught up"
/// confirmation toast without waiting for a re-fetch round-trip.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoveredFrontierAdvanceResponse {
    pub realm_id: String,
    /// New lag count after the override landed. Typically 0; non-zero
    /// means new governance Moves landed concurrently and the admin has
    /// to retry.
    #[serde(default)]
    pub lag_count: u64,
    /// Move id of the override commit.
    #[serde(default)]
    pub move_id: Option<String>,
}

impl CoveredFrontierSnapshot {
    /// Lag = number of governance Moves the MLS group has not yet
    /// acknowledged. Computed as set-difference (governance \ covered);
    /// duplicates in either side are dropped before the diff so the count
    /// is canonical. Always returns `0` when MLS is fully caught up.
    pub fn lag_count(&self) -> u64 {
        let covered: std::collections::HashSet<&str> =
            self.covered_frontier.iter().map(|s| s.as_str()).collect();
        let mut seen = std::collections::HashSet::new();
        let mut count: u64 = 0;
        for m in &self.governance_frontier {
            if covered.contains(m.as_str()) {
                continue;
            }
            if seen.insert(m.as_str()) {
                count += 1;
            }
        }
        count
    }

    /// True iff the lag is above the threshold passed in. Pure helper so
    /// the page can switch a Badge variant without re-doing the math.
    pub fn lag_above(&self, threshold: u64) -> bool {
        self.lag_count() > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lag_zero_when_covered_matches_governance() {
        let snap = CoveredFrontierSnapshot {
            realm_id: "ck:space:demo".into(),
            mls_epoch: 4,
            governance_frontier: vec!["m:1".into(), "m:2".into(), "m:3".into()],
            covered_frontier: vec!["m:3".into(), "m:1".into(), "m:2".into()],
            ..Default::default()
        };
        assert_eq!(snap.lag_count(), 0);
        assert!(!snap.lag_above(DEFAULT_LAG_WARN_THRESHOLD));
    }

    #[test]
    fn lag_counts_only_unacknowledged_moves() {
        let snap = CoveredFrontierSnapshot {
            realm_id: "ck:space:demo".into(),
            mls_epoch: 7,
            governance_frontier: vec![
                "m:1".into(),
                "m:2".into(),
                "m:3".into(),
                "m:4".into(),
                "m:5".into(),
                "m:6".into(),
                "m:7".into(),
                "m:8".into(),
            ],
            covered_frontier: vec!["m:1".into(), "m:2".into()],
            ..Default::default()
        };
        // 8 governance - 2 covered = 6 lag, above default threshold 5.
        assert_eq!(snap.lag_count(), 6);
        assert!(snap.lag_above(DEFAULT_LAG_WARN_THRESHOLD));
        assert!(!snap.lag_above(10));
    }

    #[test]
    fn lag_dedupes_duplicates_in_governance() {
        // soland normally canonicalizes the frontier, but if duplicates
        // ever leak through we must not double-count them.
        let snap = CoveredFrontierSnapshot {
            governance_frontier: vec!["m:1".into(), "m:1".into(), "m:2".into()],
            covered_frontier: vec![],
            ..Default::default()
        };
        assert_eq!(snap.lag_count(), 2);
    }

    #[test]
    fn lag_threshold_boundary_is_strict_greater_than() {
        let snap = CoveredFrontierSnapshot {
            governance_frontier: vec!["m:1".into(), "m:2".into(), "m:3".into()],
            covered_frontier: vec![],
            ..Default::default()
        };
        // lag == threshold should NOT trip the warning (strictly greater).
        assert_eq!(snap.lag_count(), 3);
        assert!(!snap.lag_above(3));
        assert!(snap.lag_above(2));
    }
}
