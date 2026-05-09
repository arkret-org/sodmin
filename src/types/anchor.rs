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
/// `soland /api/v1/admin/spaces/{id}/anchorer/reconfigure`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnchorerReconfigRequest {
    pub space_id: String,
    pub kind: String, // single_did | threshold | open_set | mixed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_n: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub threshold_dids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_set_members: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mixed_primary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mixed_recovery: Vec<String>,
}

impl AnchorerReconfigRequest {
    /// Validate spec rule: "the new anchorer cannot self-sign itself in".
    /// In other words, the admin DID submitting the reconfig Move must
    /// NOT appear as a member of the proposed anchorer set, because then
    /// the admin signature alone would suffice to install themselves as
    /// the anchorer (a privilege-escalation primitive). soland enforces
    /// this on the server too; we mirror it client-side as a pre-flight
    /// so the admin sees the constraint before signing.
    pub fn admin_self_signs_themselves_in(&self, admin_did: &str) -> bool {
        match self.kind.as_str() {
            "single_did" => self.single_did.as_deref() == Some(admin_did),
            "threshold" => self.threshold_dids.iter().any(|d| d == admin_did),
            "open_set" => self.open_set_members.iter().any(|d| d == admin_did),
            "mixed" => {
                self.mixed_primary.as_deref() == Some(admin_did)
                    || self.mixed_recovery.iter().any(|d| d == admin_did)
            }
            _ => false,
        }
    }

    /// Build the JSON body posted to soland's reconfigure endpoint. The
    /// soland handler is responsible for translating this into a real
    /// Move that targets `cx:cell:cx.component.anchorer.v1:<space>`,
    /// canonicalizing it, signing it with the admin's stored signing key
    /// (or routing it through the admin signer flow), and submitting it
    /// to the Move pipeline.
    pub fn to_reconfigure_body(&self) -> serde_json::Value {
        let mut body = serde_json::json!({
            "kind": self.kind,
        });
        if let Some(d) = &self.single_did {
            body["single_did"] = serde_json::Value::String(d.clone());
        }
        if let Some(k) = self.threshold_k {
            body["threshold_k"] = serde_json::Value::from(k);
        }
        if let Some(n) = self.threshold_n {
            body["threshold_n"] = serde_json::Value::from(n);
        }
        if !self.threshold_dids.is_empty() {
            body["threshold_dids"] =
                serde_json::Value::Array(self.threshold_dids.iter().cloned().map(Into::into).collect());
        }
        if !self.open_set_members.is_empty() {
            body["open_set_members"] = serde_json::Value::Array(
                self.open_set_members.iter().cloned().map(Into::into).collect(),
            );
        }
        if let Some(p) = &self.mixed_primary {
            body["mixed_primary"] = serde_json::Value::String(p.clone());
        }
        if !self.mixed_recovery.is_empty() {
            body["mixed_recovery"] = serde_json::Value::Array(
                self.mixed_recovery.iter().cloned().map(Into::into).collect(),
            );
        }
        body
    }
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
    /// Candidate "winner" heads for `BottomKind::Conflict` cells —
    /// concurrent Move ids the operator can pick when constructing a
    /// `head_in` repair Move. Empty for non-conflict bottoms.
    #[serde(default)]
    pub candidate_heads: Vec<WinnerHead>,
}

/// One candidate head in a Bottom-conflict repair flow. The operator picks
/// one head and the admin client builds a `head_in` repair Move that
/// re-collapses the cell to that head's value.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct WinnerHead {
    pub move_id: String,
    #[serde(default)]
    pub issuer: Option<String>,
    #[serde(default)]
    pub hlc: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
}

/// Typed strategy for resolving a Bottom cell. Mirrors the variants the
/// soland repair handler accepts; the `Manual` variant lets an operator
/// hand-craft a free-form Move payload (rare, used as the escape hatch
/// for non-conflict Bottom kinds like `SchemaError`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum BottomRepairStrategy {
    /// Pick one of the concurrent heads as the winner. soland builds a
    /// single-op `head_in` Move (per spec lattice §5.3 conflict-repair
    /// example) over the cell, signed by the admin.
    HeadInWinner { head: WinnerHead },
    /// Hand-rolled Move payload (free-form effects array) — soland MUST
    /// still verify the admin's signature & scope. Used for non-conflict
    /// Bottom kinds where there is no winning head to pick.
    Manual {
        #[serde(default)]
        note: Option<String>,
        effects: Vec<serde_json::Value>,
    },
}

impl BottomRepairStrategy {
    /// Short label for confirm-modal copy.
    pub fn label(&self) -> &'static str {
        match self {
            BottomRepairStrategy::HeadInWinner { .. } => "head_in winner",
            BottomRepairStrategy::Manual { .. } => "manual",
        }
    }
}

/// Wire body POSTed to soland's `bottom/{cell_id}/repair` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottomRepairRequest {
    pub space_id: String,
    pub cell_id: String,
    pub strategy: BottomRepairStrategy,
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

/// Body POSTed to `anchor-dag/compact`. Hint to soland how aggressively
/// to compact — `max_moves` lets the operator bound how many leaves to
/// fold into the new compaction Anchor.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompactionRequest {
    pub space_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_moves: Option<u64>,
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

    #[test]
    fn admin_self_signs_themselves_in_detects_each_kind() {
        let admin = "did:cx:admin-x";

        let single_self = AnchorerReconfigRequest {
            kind: "single_did".into(),
            single_did: Some(admin.to_string()),
            ..Default::default()
        };
        assert!(single_self.admin_self_signs_themselves_in(admin));

        let single_other = AnchorerReconfigRequest {
            kind: "single_did".into(),
            single_did: Some("did:cx:someone".into()),
            ..Default::default()
        };
        assert!(!single_other.admin_self_signs_themselves_in(admin));

        let threshold_self = AnchorerReconfigRequest {
            kind: "threshold".into(),
            threshold_dids: vec!["did:cx:a".into(), admin.to_string()],
            ..Default::default()
        };
        assert!(threshold_self.admin_self_signs_themselves_in(admin));

        let open_set_self = AnchorerReconfigRequest {
            kind: "open_set".into(),
            open_set_members: vec![admin.to_string()],
            ..Default::default()
        };
        assert!(open_set_self.admin_self_signs_themselves_in(admin));

        let mixed_recovery_self = AnchorerReconfigRequest {
            kind: "mixed".into(),
            mixed_primary: Some("did:cx:p".into()),
            mixed_recovery: vec!["did:cx:r1".into(), admin.to_string()],
            ..Default::default()
        };
        assert!(mixed_recovery_self.admin_self_signs_themselves_in(admin));
    }

    #[test]
    fn to_reconfigure_body_strips_empty_optional_fields() {
        // single_did request: should carry kind+single_did and NOT emit
        // unrelated keys (threshold_dids etc).
        let req = AnchorerReconfigRequest {
            kind: "single_did".into(),
            single_did: Some("did:cx:abc".into()),
            ..Default::default()
        };
        let body = req.to_reconfigure_body();
        assert_eq!(body.get("kind").and_then(|v| v.as_str()), Some("single_did"));
        assert_eq!(
            body.get("single_did").and_then(|v| v.as_str()),
            Some("did:cx:abc")
        );
        assert!(body.get("threshold_dids").is_none());
        assert!(body.get("open_set_members").is_none());
        assert!(body.get("mixed_primary").is_none());

        // threshold body should carry both k and n.
        let req = AnchorerReconfigRequest {
            kind: "threshold".into(),
            threshold_k: Some(2),
            threshold_n: Some(3),
            threshold_dids: vec!["did:a".into(), "did:b".into(), "did:c".into()],
            ..Default::default()
        };
        let body = req.to_reconfigure_body();
        assert_eq!(body.get("threshold_k").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(body.get("threshold_n").and_then(|v| v.as_u64()), Some(3));
        assert_eq!(
            body.get("threshold_dids").and_then(|v| v.as_array()).map(|a| a.len()),
            Some(3)
        );
    }

    #[test]
    fn bottom_repair_strategy_round_trips_through_serde() {
        let s = BottomRepairStrategy::HeadInWinner {
            head: WinnerHead {
                move_id: "move:abc".into(),
                issuer: Some("did:cx:alice".into()),
                hlc: None,
                summary: Some("set value=42".into()),
            },
        };
        let j = serde_json::to_value(&s).expect("serialize");
        // Internally tagged: strategy field must be at top level.
        assert_eq!(j.get("strategy").and_then(|v| v.as_str()), Some("head_in_winner"));
        let back: BottomRepairStrategy = serde_json::from_value(j).expect("deserialize");
        assert_eq!(back, s);
        assert_eq!(s.label(), "head_in winner");

        let m = BottomRepairStrategy::Manual {
            note: Some("schema error – manually rewrite".into()),
            effects: vec![serde_json::json!({"cell": "x", "op": {"type": "set", "value": 1}})],
        };
        let mj = serde_json::to_value(&m).expect("serialize");
        assert_eq!(mj.get("strategy").and_then(|v| v.as_str()), Some("manual"));
        assert_eq!(m.label(), "manual");
    }
}
