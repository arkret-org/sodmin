//! DTO shapes for the Move/Anchor/Lattice admin surface.
//!
//! These types mirror what soland's anchor / move / bottom admin APIs
//! return. The admin surface here is notary cells, bottom diagnostics
//! and the Seal DAG.

use serde::{Deserialize, Serialize};

// ── Anchorer cell value ──────────────────────────────────────────────────

/// Discriminator for `AnchorerValue` shapes. Matches soland's CasRegister
/// content for `ck:cell:ck.component.anchorer.v1:<realm_id>`.
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
    /// Strongly-typed anchorer kind, or `None` when the wire carried a
    /// value this build does not recognise. We deliberately do NOT fall
    /// through to `SingleDid`: a new soland anchorer kind (e.g.
    /// `hsm_backed`) must surface as "unknown" so the operator never
    /// acts on a misclassified `single_did`.
    pub fn kind(&self) -> Option<AnchorerKind> {
        match self.kind_raw.as_str() {
            "single_did" => Some(AnchorerKind::SingleDid),
            "threshold" => Some(AnchorerKind::Threshold),
            "open_set" => Some(AnchorerKind::OpenSet),
            "mixed" => Some(AnchorerKind::Mixed),
            _ => None,
        }
    }

    /// Human label for the anchorer kind, falling back to the raw wire
    /// value (prefixed) when unrecognised.
    pub fn kind_label(&self) -> String {
        match self.kind() {
            Some(k) => k.label().to_string(),
            None => format!("unknown:{}", self.kind_raw),
        }
    }

    /// One-line human summary used in lists / breadcrumbs.
    pub fn summary(&self) -> String {
        match self.kind() {
            Some(AnchorerKind::SingleDid) => {
                format!("single_did({})", self.single_did.as_deref().unwrap_or("?"))
            }
            Some(AnchorerKind::Threshold) => {
                let k = self.threshold_k.unwrap_or(0);
                let n = self.threshold_n.unwrap_or(0);
                format!("threshold({}/{})", k, n)
            }
            Some(AnchorerKind::OpenSet) => {
                format!("open_set(n={})", self.open_set_members.len())
            }
            Some(AnchorerKind::Mixed) => format!(
                "mixed(primary={}, recovery_n={})",
                self.mixed_primary.as_deref().unwrap_or("?"),
                self.mixed_recovery.len()
            ),
            None => format!("unknown({})", self.kind_raw),
        }
    }
}

// ── Anchorer reconfiguration request ─────────────────────────────────────

/// Typed reason for an `admin-self-signs-themselves-in` constraint
/// violation, surfaced per anchorer kind so the operator sees exactly
/// which sub-rule tripped.
///
/// The shared `Admin` postfix is meaningful — every variant describes
/// the same admin DID landing inside a different anchorer shape, so
/// renaming away from it would obscure intent.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfSignViolation {
    SingleDidIsAdmin,
    ThresholdContainsAdmin,
    /// Admin DID is the lex-smallest member of `threshold_dids` (the
    /// per-spec multisig §4 leader role).
    ThresholdLeaderIsAdmin,
    OpenSetContainsAdmin,
    MixedPrimaryIsAdmin,
    MixedRecoveryContainsAdmin,
}

/// Profile sent to the "construct anchorer reconfig Move" form. The
/// admin client converts this into a Move payload before POSTing to
/// `soland /_soland/admin/realms/{id}/notary/reconfigure`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnchorerReconfigRequest {
    pub realm_id: String,
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
    ///
    /// Returns true if any per-kind constraint trips. Use
    /// [`Self::self_sign_violation`] when the caller wants the typed
    /// reason (for e.g. structured error toasts).
    pub fn admin_self_signs_themselves_in(&self, admin_did: &str) -> bool {
        self.self_sign_violation(admin_did).is_some()
    }

    /// Per-kind self-sign-themselves-in constraint check, returning a
    /// typed reason when violated. Mirrors the soland-side validator.
    ///
    /// - `single_did`: `single_did != admin_did`
    /// - `threshold`:  `!threshold_dids.contains(admin_did)` AND the lex-smallest `threshold_dids`
    ///   entry (the leader per spec multisig §4) is not `admin_did`
    /// - `open_set`:   `!open_set_members.contains(admin_did)`
    /// - `mixed`:      `mixed_primary != admin_did` AND `!mixed_recovery.contains(admin_did)`
    pub fn self_sign_violation(&self, admin_did: &str) -> Option<SelfSignViolation> {
        match self.kind.as_str() {
            "single_did" => {
                if self.single_did.as_deref() == Some(admin_did) {
                    Some(SelfSignViolation::SingleDidIsAdmin)
                } else {
                    None
                }
            }
            "threshold" => {
                if self.threshold_dids.iter().any(|d| d == admin_did) {
                    return Some(SelfSignViolation::ThresholdContainsAdmin);
                }
                // Spec multisig §4: the lex-smallest member acts as the
                // leader for partial aggregation. Even if admin is not
                // in the proposed member set, refuse the case where
                // some implementation quirk would let the admin DID be
                // promoted to leader (e.g. via a future shape change).
                // For the current shape this collapses into the
                // contains() check above, but we keep it explicit so
                // the constraint is auditable per-kind.
                if let Some(leader) = self.threshold_dids.iter().min()
                    && leader == admin_did
                {
                    return Some(SelfSignViolation::ThresholdLeaderIsAdmin);
                }
                None
            }
            "open_set" => {
                if self.open_set_members.iter().any(|d| d == admin_did) {
                    Some(SelfSignViolation::OpenSetContainsAdmin)
                } else {
                    None
                }
            }
            "mixed" => {
                if self.mixed_primary.as_deref() == Some(admin_did) {
                    return Some(SelfSignViolation::MixedPrimaryIsAdmin);
                }
                if self.mixed_recovery.iter().any(|d| d == admin_did) {
                    return Some(SelfSignViolation::MixedRecoveryContainsAdmin);
                }
                None
            }
            _ => None,
        }
    }

    /// Build the JSON body posted to soland's reconfigure endpoint. The
    /// soland handler is responsible for translating this into a real
    /// Move that targets `ck:cell:ck.component.anchorer.v1:<space>`,
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
            body["threshold_dids"] = serde_json::Value::Array(
                self.threshold_dids
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect(),
            );
        }
        if !self.open_set_members.is_empty() {
            body["open_set_members"] = serde_json::Value::Array(
                self.open_set_members
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect(),
            );
        }
        if let Some(p) = &self.mixed_primary {
            body["mixed_primary"] = serde_json::Value::String(p.clone());
        }
        if !self.mixed_recovery.is_empty() {
            body["mixed_recovery"] = serde_json::Value::Array(
                self.mixed_recovery
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect(),
            );
        }
        body
    }
}

/// Response shape mirroring soland's `SubmitMoveResponse`.
///
/// `move_body` carries the canonical Move body that soland built and
/// signed on the admin's behalf — the admin UI surfaces it as a
/// readonly JSON viewer (collapsed by default) so the operator can
/// verify what was actually signed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitMoveResponse {
    pub move_id: String,
    #[serde(default)]
    pub accepted: bool,
    #[serde(default)]
    pub reason: Option<String>,
    /// Canonical signed Move body returned by soland.
    pub move_body: serde_json::Value,
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
    pub realm_id: String,
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

/// Body POSTed to `seal-dag/compact`. Hint to soland how aggressively
/// to compact — `max_moves` lets the operator bound how many leaves to
/// fold into the new compaction Anchor.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompactionRequest {
    pub realm_id: String,
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
        assert_eq!(v.kind(), Some(AnchorerKind::Threshold));
        assert_eq!(v.summary(), "threshold(2/3)");
    }

    #[test]
    fn anchorer_value_summary_single_did_default() {
        let v = AnchorerValue {
            kind_raw: "single_did".into(),
            single_did: Some("did:ck:abc".into()),
            ..Default::default()
        };
        assert_eq!(v.kind(), Some(AnchorerKind::SingleDid));
        assert!(v.summary().contains("did:ck:abc"));
        // Unknown kind_raw surfaces as `None` (not a misclassified
        // single_did) so the UI renders an explicit "unknown" badge.
        let unknown = AnchorerValue {
            kind_raw: "garbage".into(),
            ..Default::default()
        };
        assert_eq!(unknown.kind(), None);
        assert_eq!(unknown.kind_label(), "unknown:garbage");
        assert_eq!(unknown.summary(), "unknown(garbage)");
    }

    #[test]
    fn anchorer_value_summary_open_set_and_mixed() {
        let open = AnchorerValue {
            kind_raw: "open_set".into(),
            open_set_members: vec!["did:1".into(), "did:2".into()],
            ..Default::default()
        };
        assert_eq!(open.kind(), Some(AnchorerKind::OpenSet));
        assert_eq!(open.summary(), "open_set(n=2)");

        let mixed = AnchorerValue {
            kind_raw: "mixed".into(),
            mixed_primary: Some("did:p".into()),
            mixed_recovery: vec!["did:r1".into(), "did:r2".into(), "did:r3".into()],
            ..Default::default()
        };
        assert_eq!(mixed.kind(), Some(AnchorerKind::Mixed));
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
    fn anchorer_kind_label_round_trip() {
        assert_eq!(AnchorerKind::SingleDid.label(), "single_did");
        assert_eq!(AnchorerKind::Threshold.label(), "threshold");
        assert_eq!(AnchorerKind::OpenSet.label(), "open_set");
        assert_eq!(AnchorerKind::Mixed.label(), "mixed");
    }

    #[test]
    fn admin_self_signs_themselves_in_detects_each_kind() {
        let admin = "did:ck:admin-x";

        let single_self = AnchorerReconfigRequest {
            kind: "single_did".into(),
            single_did: Some(admin.to_string()),
            ..Default::default()
        };
        assert!(single_self.admin_self_signs_themselves_in(admin));

        let single_other = AnchorerReconfigRequest {
            kind: "single_did".into(),
            single_did: Some("did:ck:someone".into()),
            ..Default::default()
        };
        assert!(!single_other.admin_self_signs_themselves_in(admin));

        let threshold_self = AnchorerReconfigRequest {
            kind: "threshold".into(),
            threshold_dids: vec!["did:ck:a".into(), admin.to_string()],
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
            mixed_primary: Some("did:ck:p".into()),
            mixed_recovery: vec!["did:ck:r1".into(), admin.to_string()],
            ..Default::default()
        };
        assert!(mixed_recovery_self.admin_self_signs_themselves_in(admin));
    }

    // ── Per-anchorer-kind self-sign-themselves-in unit tests (round 27) ──
    //
    // Each test pins the typed `SelfSignViolation` reason returned per
    // kind so the soland-side parity is auditable from one place.

    #[test]
    fn self_sign_violation_single_did_kind() {
        let admin = "did:ck:admin-x";
        // Positive: admin is the proposed single_did.
        let bad = AnchorerReconfigRequest {
            kind: "single_did".into(),
            single_did: Some(admin.to_string()),
            ..Default::default()
        };
        assert_eq!(
            bad.self_sign_violation(admin),
            Some(SelfSignViolation::SingleDidIsAdmin)
        );
        assert!(bad.admin_self_signs_themselves_in(admin));

        // Negative: a different DID is fine.
        let ok = AnchorerReconfigRequest {
            kind: "single_did".into(),
            single_did: Some("did:ck:other".into()),
            ..Default::default()
        };
        assert_eq!(ok.self_sign_violation(admin), None);
        assert!(!ok.admin_self_signs_themselves_in(admin));
    }

    #[test]
    fn self_sign_violation_threshold_kind() {
        let admin = "did:ck:admin-x";

        // Positive: admin is anywhere in the threshold member set —
        // this is the broader contains() check that fires first.
        let contains = AnchorerReconfigRequest {
            kind: "threshold".into(),
            threshold_k: Some(2),
            threshold_n: Some(3),
            threshold_dids: vec!["did:ck:b".into(), "did:ck:c".into(), admin.to_string()],
            ..Default::default()
        };
        assert_eq!(
            contains.self_sign_violation(admin),
            Some(SelfSignViolation::ThresholdContainsAdmin)
        );

        // Positive: admin IS the lex-smallest leader. The contains()
        // arm trips first, but the constraint still classifies it as a
        // self-sign violation.
        let leader = AnchorerReconfigRequest {
            kind: "threshold".into(),
            threshold_k: Some(2),
            threshold_n: Some(3),
            threshold_dids: vec![
                admin.to_string(), // lex-smallest because "admin-x" < "b"
                "did:ck:b".into(),
                "did:ck:c".into(),
            ],
            ..Default::default()
        };
        assert_eq!(
            leader.self_sign_violation(admin),
            Some(SelfSignViolation::ThresholdContainsAdmin)
        );

        // Negative: admin not in member set, no violation.
        let ok = AnchorerReconfigRequest {
            kind: "threshold".into(),
            threshold_k: Some(2),
            threshold_n: Some(3),
            threshold_dids: vec!["did:ck:a".into(), "did:ck:b".into(), "did:ck:c".into()],
            ..Default::default()
        };
        assert_eq!(ok.self_sign_violation(admin), None);
    }

    #[test]
    fn self_sign_violation_open_set_kind() {
        let admin = "did:ck:admin-x";

        // Positive: admin is in the open-set members.
        let bad = AnchorerReconfigRequest {
            kind: "open_set".into(),
            open_set_members: vec!["did:ck:a".into(), admin.to_string()],
            ..Default::default()
        };
        assert_eq!(
            bad.self_sign_violation(admin),
            Some(SelfSignViolation::OpenSetContainsAdmin)
        );

        // Negative: admin not in member set.
        let ok = AnchorerReconfigRequest {
            kind: "open_set".into(),
            open_set_members: vec!["did:ck:a".into(), "did:ck:b".into()],
            ..Default::default()
        };
        assert_eq!(ok.self_sign_violation(admin), None);
    }

    #[test]
    fn self_sign_violation_mixed_kind() {
        let admin = "did:ck:admin-x";

        // Positive (primary): admin is the primary DID.
        let primary = AnchorerReconfigRequest {
            kind: "mixed".into(),
            mixed_primary: Some(admin.to_string()),
            mixed_recovery: vec!["did:ck:r1".into(), "did:ck:r2".into()],
            ..Default::default()
        };
        assert_eq!(
            primary.self_sign_violation(admin),
            Some(SelfSignViolation::MixedPrimaryIsAdmin)
        );

        // Positive (recovery): admin appears in recovery quorum.
        let recovery = AnchorerReconfigRequest {
            kind: "mixed".into(),
            mixed_primary: Some("did:ck:p".into()),
            mixed_recovery: vec!["did:ck:r1".into(), admin.to_string(), "did:ck:r2".into()],
            ..Default::default()
        };
        assert_eq!(
            recovery.self_sign_violation(admin),
            Some(SelfSignViolation::MixedRecoveryContainsAdmin)
        );

        // Negative: clean primary + recovery quorum.
        let ok = AnchorerReconfigRequest {
            kind: "mixed".into(),
            mixed_primary: Some("did:ck:p".into()),
            mixed_recovery: vec!["did:ck:r1".into(), "did:ck:r2".into()],
            ..Default::default()
        };
        assert_eq!(ok.self_sign_violation(admin), None);
    }

    #[test]
    fn to_reconfigure_body_strips_empty_optional_fields() {
        // single_did request: should carry kind+single_did and NOT emit
        // unrelated keys (threshold_dids etc).
        let req = AnchorerReconfigRequest {
            kind: "single_did".into(),
            single_did: Some("did:ck:abc".into()),
            ..Default::default()
        };
        let body = req.to_reconfigure_body();
        assert_eq!(
            body.get("kind").and_then(|v| v.as_str()),
            Some("single_did")
        );
        assert_eq!(
            body.get("single_did").and_then(|v| v.as_str()),
            Some("did:ck:abc")
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
            body.get("threshold_dids")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(3)
        );
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

    // ── H'3 round 27 — explicit round-trip tests for both
    //    BottomRepairStrategy variants. These verify that the
    //    snake_case tag discriminator + the per-variant field shape
    //    survives a full `to_string` → `from_str` round-trip (string,
    //    not Value), so the wire bytes the soland handler observes
    //    match what the admin client serialised.

    #[test]
    fn bottom_repair_strategy_head_in_winner_round_trip_through_serde_string() {
        let s = BottomRepairStrategy::HeadInWinner {
            head: WinnerHead {
                move_id: "sha256:deadbeef".into(),
                issuer: Some("did:ck:alice".into()),
                hlc: Some("01HXY-0001".into()),
                summary: Some("set value=99".into()),
            },
        };
        let wire = serde_json::to_string(&s).expect("serialize to string");
        // Snake-case tag + tag key survives.
        assert!(wire.contains("\"strategy\":\"head_in_winner\""));
        // Fields inside the variant render as snake_case too.
        assert!(wire.contains("\"move_id\":\"sha256:deadbeef\""));
        assert!(wire.contains("\"issuer\":\"did:ck:alice\""));
        assert!(wire.contains("\"hlc\":\"01HXY-0001\""));
        let back: BottomRepairStrategy =
            serde_json::from_str(&wire).expect("deserialize from string");
        assert_eq!(back, s);
        assert_eq!(back.label(), "head_in winner");
    }

    #[test]
    fn bottom_repair_strategy_manual_round_trip_through_serde_string() {
        let m = BottomRepairStrategy::Manual {
            note: Some("schema error - hand-rewrite the cell".into()),
            effects: vec![
                serde_json::json!({
                    "cell": "ck:cell:ck.component.x.v1:demo",
                    "op": {"type": "cas_register", "value": {"foo": 1}},
                }),
                serde_json::json!({
                    "cell": "ck:cell:ck.component.y.v1:demo",
                    "op": {"type": "set_membership_add", "value": "did:ck:carol"},
                }),
            ],
        };
        let wire = serde_json::to_string(&m).expect("serialize to string");
        // Tag survives + is snake_case.
        assert!(wire.contains("\"strategy\":\"manual\""));
        // Variant fields render as snake_case.
        assert!(wire.contains("\"note\":\"schema error - hand-rewrite the cell\""));
        assert!(wire.contains("\"effects\""));
        let back: BottomRepairStrategy =
            serde_json::from_str(&wire).expect("deserialize from string");
        assert_eq!(back, m);
        assert_eq!(back.label(), "manual");
        // And the effects array survives byte-for-byte (ordering +
        // length).
        if let BottomRepairStrategy::Manual { effects, note } = back {
            assert_eq!(effects.len(), 2);
            assert_eq!(
                note.as_deref(),
                Some("schema error - hand-rewrite the cell"),
            );
        } else {
            panic!("expected Manual variant after round-trip");
        }
    }
}
