//! DTO shapes for the Move/Seal/Lattice admin surface.
//!
//! These types mirror what soland's seal / move / bottom admin APIs
//! return. The admin surface here is notary cells, bottom diagnostics
//! and the Seal DAG.
//!
//! The notary cell value wire shape is the SDK-authoritative
//! [`cokret_core::NotaryValue`] (internally tagged on `kind`, fields
//! `did|k|n|members|primary|recovery_members`) — sodmin carries no
//! projection type for it.

use serde::{Deserialize, Serialize};

// ── Notary cell value ────────────────────────────────────────────────────

// SDK-authoritative notary cell value for
// `ck:cell:ck.component.notary.v1:<realm_id>` (internal tag `kind`:
// `single_did{did} | threshold{k,n,members} | open_set{members} |
// mixed{primary,recovery_members}`).
pub use cokret_core::NotaryValue;

/// `GET /_soland/admin/realms/{realm_id}/notary` response: the
/// authoritative [`NotaryValue`] flattened at the top level plus the
/// envelope hints soland projects alongside it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotaryCellValue {
    #[serde(flatten)]
    pub value: NotaryValue,
    #[serde(default)]
    pub revocation_freshness_window_ms: Option<u64>,
    #[serde(default)]
    pub paused: bool,
}

/// Display helpers for the SDK [`NotaryValue`].
pub trait NotaryValueExt {
    /// Wire discriminator (`single_did|threshold|open_set|mixed`).
    fn kind_label(&self) -> &'static str;
    /// One-line human summary used in lists / breadcrumbs.
    fn summary(&self) -> String;
}

impl NotaryValueExt for NotaryValue {
    fn kind_label(&self) -> &'static str {
        match self {
            NotaryValue::SingleDid { .. } => "single_did",
            NotaryValue::Threshold { .. } => "threshold",
            NotaryValue::OpenSet { .. } => "open_set",
            NotaryValue::Mixed { .. } => "mixed",
        }
    }

    fn summary(&self) -> String {
        match self {
            NotaryValue::SingleDid { did } => format!("single_did({did})"),
            NotaryValue::Threshold { k, n, .. } => format!("threshold({k}/{n})"),
            NotaryValue::OpenSet { members } => format!("open_set(n={})", members.len()),
            NotaryValue::Mixed {
                primary,
                recovery_members,
            } => format!(
                "mixed(primary={primary}, recovery_n={})",
                recovery_members.len()
            ),
        }
    }
}

// ── Notary reconfiguration request ───────────────────────────────────────

/// Typed reason for an `admin-self-signs-themselves-in` constraint
/// violation, surfaced per notary kind so the operator sees exactly
/// which sub-rule tripped.
///
/// The shared `Admin` postfix is meaningful — every variant describes
/// the same admin DID landing inside a different notary shape, so
/// renaming away from it would obscure intent.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfSignViolation {
    SingleDidIsAdmin,
    ThresholdContainsAdmin,
    /// Admin DID is the lex-smallest threshold member (the per-spec
    /// multisig §4 leader role).
    ThresholdLeaderIsAdmin,
    OpenSetContainsAdmin,
    MixedPrimaryIsAdmin,
    MixedRecoveryContainsAdmin,
}

/// Validate spec rule: "the new notary cannot self-sign itself in".
/// The admin DID submitting the reconfig Move must NOT appear as a
/// member of the proposed notary set, because then the admin signature
/// alone would suffice to install themselves as the notary (a
/// privilege-escalation primitive). soland enforces this on the server
/// too; we mirror it client-side as a pre-flight so the admin sees the
/// constraint before signing.
///
/// - `single_did`: `did != admin_did`
/// - `threshold`:  `!members.contains(admin_did)` AND the lex-smallest member (the leader per spec
///   multisig §4) is not `admin_did`
/// - `open_set`:   `!members.contains(admin_did)`
/// - `mixed`:      `primary != admin_did` AND `!recovery_members.contains(admin_did)`
pub fn self_sign_violation(value: &NotaryValue, admin_did: &str) -> Option<SelfSignViolation> {
    match value {
        NotaryValue::SingleDid { did } => {
            if did.as_str() == admin_did {
                Some(SelfSignViolation::SingleDidIsAdmin)
            } else {
                None
            }
        }
        NotaryValue::Threshold { members, .. } => {
            if members.iter().any(|d| d.as_str() == admin_did) {
                return Some(SelfSignViolation::ThresholdContainsAdmin);
            }
            // Spec multisig §4: the lex-smallest member acts as the
            // leader for partial aggregation. For the current shape this
            // collapses into the contains() check above, but we keep it
            // explicit so the constraint is auditable per-kind.
            if let Some(leader) = members.iter().map(|d| d.as_str()).min()
                && leader == admin_did
            {
                return Some(SelfSignViolation::ThresholdLeaderIsAdmin);
            }
            None
        }
        NotaryValue::OpenSet { members } => {
            if members.iter().any(|d| d.as_str() == admin_did) {
                Some(SelfSignViolation::OpenSetContainsAdmin)
            } else {
                None
            }
        }
        NotaryValue::Mixed {
            primary,
            recovery_members,
        } => {
            if primary.as_str() == admin_did {
                return Some(SelfSignViolation::MixedPrimaryIsAdmin);
            }
            if recovery_members.iter().any(|d| d.as_str() == admin_did) {
                return Some(SelfSignViolation::MixedRecoveryContainsAdmin);
            }
            None
        }
    }
}

/// Convenience boolean wrapper over [`self_sign_violation`].
pub fn admin_self_signs_themselves_in(value: &NotaryValue, admin_did: &str) -> bool {
    self_sign_violation(value, admin_did).is_some()
}

/// Response shape mirroring soland's `AdminSubmitMoveOutcome`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitMoveOutcome {
    pub move_id: String,
    #[serde(default)]
    pub accepted: bool,
    #[serde(default)]
    pub reason: Option<String>,
    /// Seal id when the admin Move was already folded into a fresh Seal
    /// by the in-process notary worker.
    #[serde(default)]
    pub seal_id: Option<String>,
    /// `pending|accepted|rejected|scope_validated|placeholder`.
    #[serde(default)]
    pub status: String,
}

// ── Bottom diagnostics ───────────────────────────────────────────────────

// SDK-authoritative Bottom kind (`bottom.schema.json` — six normative
// variants; receivers MUST fail closed on unrecognized kinds, hence the
// `Option` in [`bottom_kind_from_wire`]).
pub use cokret_core::BottomKind;

/// Display helpers for the SDK [`BottomKind`].
pub trait BottomKindExt {
    fn label(&self) -> &'static str;
}

impl BottomKindExt for BottomKind {
    fn label(&self) -> &'static str {
        match self {
            BottomKind::Conflict => "Conflict",
            BottomKind::InvalidTransition => "Invalid Transition",
            BottomKind::MissingDependency => "Missing Dependency",
            BottomKind::Unauthorized => "Unauthorized",
            BottomKind::NotarySplit => "Notary Split",
            BottomKind::SchemaError => "Schema Error",
        }
    }
}

/// Parse a [`BottomKind`] from the wire enum string. `None` for
/// unrecognized kinds (fail closed — never misclassify a new kind).
pub fn bottom_kind_from_wire(s: &str) -> Option<BottomKind> {
    serde_json::from_value(serde_json::Value::String(s.to_owned())).ok()
}

/// One row in the bottom diagnostics view.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BottomEntry {
    pub realm_id: String,
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
    pub realm_id: String,
    pub cell_id: String,
    pub strategy: BottomRepairStrategy,
}

// ── Seal DAG ─────────────────────────────────────────────────────────────

/// One Seal DAG leaf row. Mirrors soland's `SealLeafOutcome`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SealLeaf {
    pub seal_id: String,
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

/// `GET .../seal-dag` response. Mirrors soland's `SealDagSnapshotOutcome`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SealDagSnapshot {
    pub realm_id: String,
    pub leaves: Vec<SealLeaf>,
    #[serde(default)]
    pub covered_event_digests: Vec<String>,
    #[serde(default)]
    pub state_root: Option<String>,
    #[serde(default)]
    pub last_compaction_at: Option<String>,
}

/// `POST .../seal-dag/compact` response. Mirrors soland's
/// `CompactionOutcome`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompactionOutcome {
    pub seal_id: String,
    #[serde(default)]
    pub state_root: Option<String>,
    #[serde(default)]
    pub move_count: u64,
}

/// Body POSTed to `seal-dag/compact`. Hint to soland how aggressively
/// to compact — `max_moves` lets the operator bound how many Moves to
/// fold into the new compaction Seal.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompactionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_moves: Option<u64>,
}

#[cfg(test)]
mod tests {
    use cokret_core::Did;
    use serde_json::json;

    use super::*;

    fn did(s: &str) -> Did {
        Did::new(s.to_owned()).expect("valid did")
    }

    #[test]
    fn notary_value_wire_shape_is_sdk_authoritative() {
        // Internal tag `kind` + canonical field names — no flat alias
        // spellings (`kind_raw`/`threshold_dids`/...).
        let v = NotaryValue::Threshold {
            k: 2,
            n: 3,
            members: vec![did("did:ck:a"), did("did:ck:b"), did("did:ck:c")],
        };
        let j = serde_json::to_value(&v).expect("serialize");
        assert_eq!(j["kind"], "threshold");
        assert_eq!(j["k"], 2);
        assert_eq!(j["n"], 3);
        assert_eq!(j["members"].as_array().map(|a| a.len()), Some(3));
        assert!(j.get("threshold_k").is_none());
        assert!(j.get("threshold_dids").is_none());

        // Legacy flat spellings no longer parse.
        assert!(
            serde_json::from_value::<NotaryValue>(json!({
                "kind_raw": "threshold",
                "threshold_dids": ["did:ck:a"],
            }))
            .is_err()
        );
    }

    #[test]
    fn notary_cell_value_parses_flattened_envelope() {
        let cell: NotaryCellValue = serde_json::from_value(json!({
            "kind": "single_did",
            "did": "did:ck:abc",
            "revocation_freshness_window_ms": 60000,
            "paused": true,
        }))
        .expect("parse");
        assert_eq!(cell.value.kind_label(), "single_did");
        assert_eq!(cell.revocation_freshness_window_ms, Some(60000));
        assert!(cell.paused);
    }

    #[test]
    fn notary_value_summaries() {
        let single = NotaryValue::SingleDid {
            did: did("did:ck:abc"),
        };
        assert_eq!(single.kind_label(), "single_did");
        assert!(single.summary().contains("did:ck:abc"));

        let threshold = NotaryValue::Threshold {
            k: 2,
            n: 3,
            members: vec![did("did:ck:a"), did("did:ck:b"), did("did:ck:c")],
        };
        assert_eq!(threshold.summary(), "threshold(2/3)");

        let open = NotaryValue::OpenSet {
            members: vec![did("did:ck:1"), did("did:ck:2")],
        };
        assert_eq!(open.summary(), "open_set(n=2)");

        let mixed = NotaryValue::Mixed {
            primary: did("did:ck:p"),
            recovery_members: vec![did("did:ck:r1"), did("did:ck:r2"), did("did:ck:r3")],
        };
        let s = mixed.summary();
        assert!(s.contains("did:ck:p"));
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
            ("notary_split", "Notary Split"),
            ("schema_error", "Schema Error"),
        ] {
            let k = bottom_kind_from_wire(wire).expect("wire variant parses");
            assert_eq!(k.label(), expected_label);
        }
        assert!(bottom_kind_from_wire("CONFLICT").is_none());
        assert!(bottom_kind_from_wire("not_a_kind").is_none());
        // anchorer_split was the pre-neutralization name; the SDK enum is
        // authoritative and only accepts notary_split.
        assert!(bottom_kind_from_wire("anchorer_split").is_none());
    }

    #[test]
    fn self_sign_violation_detects_each_kind() {
        let admin = "did:ck:admin-x";

        let single_self = NotaryValue::SingleDid { did: did(admin) };
        assert_eq!(
            self_sign_violation(&single_self, admin),
            Some(SelfSignViolation::SingleDidIsAdmin)
        );
        assert!(admin_self_signs_themselves_in(&single_self, admin));

        let single_other = NotaryValue::SingleDid {
            did: did("did:ck:someone"),
        };
        assert_eq!(self_sign_violation(&single_other, admin), None);

        let threshold_self = NotaryValue::Threshold {
            k: 2,
            n: 2,
            members: vec![did("did:ck:a"), did(admin)],
        };
        assert_eq!(
            self_sign_violation(&threshold_self, admin),
            Some(SelfSignViolation::ThresholdContainsAdmin)
        );

        let threshold_ok = NotaryValue::Threshold {
            k: 2,
            n: 3,
            members: vec![did("did:ck:a"), did("did:ck:b"), did("did:ck:c")],
        };
        assert_eq!(self_sign_violation(&threshold_ok, admin), None);

        let open_set_self = NotaryValue::OpenSet {
            members: vec![did(admin)],
        };
        assert_eq!(
            self_sign_violation(&open_set_self, admin),
            Some(SelfSignViolation::OpenSetContainsAdmin)
        );

        let mixed_primary_self = NotaryValue::Mixed {
            primary: did(admin),
            recovery_members: vec![did("did:ck:r1")],
        };
        assert_eq!(
            self_sign_violation(&mixed_primary_self, admin),
            Some(SelfSignViolation::MixedPrimaryIsAdmin)
        );

        let mixed_recovery_self = NotaryValue::Mixed {
            primary: did("did:ck:p"),
            recovery_members: vec![did("did:ck:r1"), did(admin)],
        };
        assert_eq!(
            self_sign_violation(&mixed_recovery_self, admin),
            Some(SelfSignViolation::MixedRecoveryContainsAdmin)
        );

        let mixed_ok = NotaryValue::Mixed {
            primary: did("did:ck:p"),
            recovery_members: vec![did("did:ck:r1"), did("did:ck:r2")],
        };
        assert_eq!(self_sign_violation(&mixed_ok, admin), None);
    }

    #[test]
    fn bottom_repair_strategy_round_trips_through_serde() {
        let s = BottomRepairStrategy::HeadInWinner {
            head: WinnerHead {
                move_id: "move:abc".into(),
                issuer: Some("did:ck:alice".into()),
                hlc: None,
                summary: Some("set value=42".into()),
            },
        };
        let j = serde_json::to_value(&s).expect("serialize");
        // Internally tagged: strategy field must be at top level.
        assert_eq!(
            j.get("strategy").and_then(|v| v.as_str()),
            Some("head_in_winner")
        );
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

    #[test]
    fn seal_dag_shapes_match_soland_wire() {
        // Field names pin soland's `SealLeafOutcome` /
        // `SealDagSnapshotOutcome` / `CompactionOutcome` (seal_id, NOT the
        // pre-rename anchor_id; covered_event_digests, NOT frontier).
        let snapshot: SealDagSnapshot = serde_json::from_value(json!({
            "realm_id": "ck:realm:demo",
            "leaves": [{
                "seal_id": "ck:seal:sha256:aa",
                "state_root": "sha256:bb",
                "move_count": 3,
                "signers": ["did:web:soland.local#notary-key"],
                "is_compaction": false,
            }],
            "covered_event_digests": ["sha256:cc"],
            "state_root": "sha256:bb",
        }))
        .expect("parse");
        assert_eq!(snapshot.leaves[0].seal_id, "ck:seal:sha256:aa");
        assert_eq!(snapshot.covered_event_digests.len(), 1);

        let outcome: CompactionOutcome = serde_json::from_value(json!({
            "seal_id": "ck:seal:sha256:dd",
            "move_count": 0,
        }))
        .expect("parse");
        assert_eq!(outcome.seal_id, "ck:seal:sha256:dd");

        let body = CompactionRequest { max_moves: None };
        let s = serde_json::to_string(&body).expect("serialize");
        assert!(!s.contains("max_moves"));
    }

    #[test]
    fn submit_move_outcome_matches_soland_wire() {
        let r: SubmitMoveOutcome = serde_json::from_value(json!({
            "move_id": "sha256:00",
            "accepted": true,
            "seal_id": "ck:seal:sha256:ee",
            "status": "accepted",
        }))
        .expect("parse");
        assert_eq!(r.status, "accepted");
        assert_eq!(r.seal_id.as_deref(), Some("ck:seal:sha256:ee"));
    }
}
