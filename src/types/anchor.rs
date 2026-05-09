//! DTO shapes for the Move/Anchor/Lattice admin surface (Stream H', C10.F).
//!
//! These types mirror what soland's anchor / move / bottom admin APIs are
//! expected to return; they are intentionally hand-written here as an interim
//! step. Once `soland-admin-types` lands they will be replaced by
//! `pub use soland_admin_types::*;` aliases — see `_todos.md` A0 checklist.
//!
//! The old hub-Space admin shapes (`writer_model`, `host_endorsement`,
//! `host_transfer`) were deleted by `contrix-spec` 2026-05-08 and have no
//! replacements here — the admin surface is now anchorer cells, bottom
//! diagnostics and the Anchor DAG.

use serde::{Deserialize, Serialize};

// ── Anchorer cell value ──────────────────────────────────────────────────

/// Discriminator for `AnchorerValue` shapes. Matches soland's CasRegister
/// content for `cx:cell:cx.component.anchorer.v1:<space_id>`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnchorerKind {
    SingleDid,
    Threshold,
    OpenSet,
    Mixed,
}

impl AnchorerKind {
    pub fn label(&self) -> &'static str {
        match self {
            AnchorerKind::SingleDid => "single_did",
            AnchorerKind::Threshold => "threshold",
            AnchorerKind::OpenSet => "open_set",
            AnchorerKind::Mixed => "mixed",
        }
    }
}

/// Read-only view of an anchorer cell's current value, joined and projected
/// by soland.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnchorerValue {
    pub kind_raw: String,
    /// Populated when `kind_raw == "single_did"`.
    #[serde(default)]
    pub single_did: Option<String>,
    /// Populated when `kind_raw == "threshold"`.
    #[serde(default)]
    pub threshold_k: Option<u32>,
    #[serde(default)]
    pub threshold_n: Option<u32>,
    #[serde(default)]
    pub threshold_dids: Vec<String>,
    /// Populated when `kind_raw == "open_set"`.
    #[serde(default)]
    pub open_set_members: Vec<String>,
    /// Populated when `kind_raw == "mixed"`. Primary anchorer DID.
    #[serde(default)]
    pub mixed_primary: Option<String>,
    /// Populated when `kind_raw == "mixed"`. Recovery quorum DIDs.
    #[serde(default)]
    pub mixed_recovery: Vec<String>,
    #[serde(default)]
    pub max_anchor_staleness_ms: Option<u64>,
    #[serde(default)]
    pub paused: bool,
}

impl AnchorerValue {
    pub fn kind(&self) -> AnchorerKind {
        match self.kind_raw.as_str() {
            "threshold" => AnchorerKind::Threshold,
            "open_set" => AnchorerKind::OpenSet,
            "mixed" => AnchorerKind::Mixed,
            _ => AnchorerKind::SingleDid,
        }
    }

    /// One-line human summary used in lists / breadcrumbs.
    pub fn summary(&self) -> String {
        match self.kind() {
            AnchorerKind::SingleDid => {
                format!("single_did({})", self.single_did.as_deref().unwrap_or("?"))
            }
            AnchorerKind::Threshold => {
                let k = self.threshold_k.unwrap_or(0);
                let n = self.threshold_n.unwrap_or(0);
                format!("threshold({}/{})", k, n)
            }
            AnchorerKind::OpenSet => {
                format!("open_set(n={})", self.open_set_members.len())
            }
            AnchorerKind::Mixed => format!(
                "mixed(primary={}, recovery_n={})",
                self.mixed_primary.as_deref().unwrap_or("?"),
                self.mixed_recovery.len()
            ),
        }
    }
}

// ── Anchorer reconfiguration request ─────────────────────────────────────

/// Profile sent to the "construct anchorer reconfig Move" form. The
/// admin client converts this into a Move payload before POSTing to
/// `soland /api/v1/moves`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnchorerReconfigRequest {
    pub space_id: String,
    pub kind: String, // single_did | threshold | open_set | mixed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_k: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub threshold_dids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_set_members: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mixed_primary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mixed_recovery: Vec<String>,
}

/// Response shape mirroring soland's `SubmitMoveResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitMoveResponse {
    pub move_id: String,
    #[serde(default)]
    pub accepted: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

// ── Bottom diagnostics ───────────────────────────────────────────────────

/// Structured Bottom kind exposed by the lattice when `join` produces a
/// `Bottom`. Matches the soland reducer enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BottomKind {
    Conflict,
    InvalidTransition,
    MissingDependency,
    Unauthorized,
    AnchorerSplit,
    SchemaError,
}

impl BottomKind {
    pub fn label(&self) -> &'static str {
        match self {
            BottomKind::Conflict => "Conflict",
            BottomKind::InvalidTransition => "Invalid Transition",
            BottomKind::MissingDependency => "Missing Dependency",
            BottomKind::Unauthorized => "Unauthorized",
            BottomKind::AnchorerSplit => "Anchorer Split",
            BottomKind::SchemaError => "Schema Error",
        }
    }

    /// Parse from the wire enum string.
    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "conflict" => Some(BottomKind::Conflict),
            "invalid_transition" => Some(BottomKind::InvalidTransition),
            "missing_dependency" => Some(BottomKind::MissingDependency),
            "unauthorized" => Some(BottomKind::Unauthorized),
            "anchorer_split" => Some(BottomKind::AnchorerSplit),
            "schema_error" => Some(BottomKind::SchemaError),
            _ => None,
        }
    }
}

/// One row in the bottom diagnostics view.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BottomEntry {
    pub space_id: String,
    pub cell_id: String,
    pub kind: String, // wire-format BottomKind
    #[serde(default)]
    pub move_ids: Vec<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub detected_at: Option<String>,
}

// ── Anchor DAG ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnchorLeaf {
    pub anchor_id: String,
    #[serde(default)]
    pub state_root: Option<String>,
    #[serde(default)]
    pub move_count: u64,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub signers: Vec<String>,
    #[serde(default)]
    pub is_compaction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnchorDagSnapshot {
    pub space_id: String,
    pub leaves: Vec<AnchorLeaf>,
    pub frontier: Vec<String>,
    #[serde(default)]
    pub state_root: Option<String>,
    #[serde(default)]
    pub last_compaction_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignAnchorResponse {
    pub anchor_id: String,
    #[serde(default)]
    pub state_root: Option<String>,
    #[serde(default)]
    pub move_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchorer_value_summary_threshold() {
        let v = AnchorerValue {
            kind_raw: "threshold".into(),
            threshold_k: Some(2),
            threshold_n: Some(3),
            threshold_dids: vec!["did:a".into(), "did:b".into(), "did:c".into()],
            ..Default::default()
        };
        assert_eq!(v.kind(), AnchorerKind::Threshold);
        assert_eq!(v.summary(), "threshold(2/3)");
    }

    #[test]
    fn anchorer_value_summary_single_did_default() {
        let v = AnchorerValue {
            kind_raw: "single_did".into(),
            single_did: Some("did:cx:abc".into()),
            ..Default::default()
        };
        assert_eq!(v.kind(), AnchorerKind::SingleDid);
        assert!(v.summary().contains("did:cx:abc"));
        // Unknown kind_raw collapses to single_did so the UI never panics.
        let unknown = AnchorerValue {
            kind_raw: "garbage".into(),
            ..Default::default()
        };
        assert_eq!(unknown.kind(), AnchorerKind::SingleDid);
    }

    #[test]
    fn anchorer_value_summary_open_set_and_mixed() {
        let open = AnchorerValue {
            kind_raw: "open_set".into(),
            open_set_members: vec!["did:1".into(), "did:2".into()],
            ..Default::default()
        };
        assert_eq!(open.kind(), AnchorerKind::OpenSet);
        assert_eq!(open.summary(), "open_set(n=2)");

        let mixed = AnchorerValue {
            kind_raw: "mixed".into(),
            mixed_primary: Some("did:p".into()),
            mixed_recovery: vec!["did:r1".into(), "did:r2".into(), "did:r3".into()],
            ..Default::default()
        };
        assert_eq!(mixed.kind(), AnchorerKind::Mixed);
        let s = mixed.summary();
        assert!(s.contains("did:p"));
        assert!(s.contains("recovery_n=3"));
    }

    #[test]
    fn bottom_kind_round_trip_and_label() {
        // All wire variants parse back, and parsing is case-sensitive
        // snake_case (rejecting random strings).
        for (wire, expected_label) in [
            ("conflict", "Conflict"),
            ("invalid_transition", "Invalid Transition"),
            ("missing_dependency", "Missing Dependency"),
            ("unauthorized", "Unauthorized"),
            ("anchorer_split", "Anchorer Split"),
            ("schema_error", "Schema Error"),
        ] {
            let k = BottomKind::from_wire(wire).expect("wire variant parses");
            assert_eq!(k.label(), expected_label);
        }
        assert!(BottomKind::from_wire("CONFLICT").is_none());
        assert!(BottomKind::from_wire("not_a_kind").is_none());
    }

    #[test]
    fn anchorer_kind_label_round_trip() {
        assert_eq!(AnchorerKind::SingleDid.label(), "single_did");
        assert_eq!(AnchorerKind::Threshold.label(), "threshold");
        assert_eq!(AnchorerKind::OpenSet.label(), "open_set");
        assert_eq!(AnchorerKind::Mixed.label(), "mixed");
    }
}
