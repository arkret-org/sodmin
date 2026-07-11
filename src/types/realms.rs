//! DTO shapes for the Realm admin surface.

use arkret_core::{BlobRef, Discoverability, HistoryVisibility, JoinRule};
use serde::{Deserialize, Serialize};

use crate::types::HandleClaim;

// ── Realm types (security boundary) ──
//
// The object that carries encryption / join-rule / history-visibility /
// realm-class boundary fields is a Realm. Space containers are represented
// separately by `spaces::SpaceRow`.

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminRealm {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub discoverability: Option<Discoverability>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub member_count: u64,
    #[serde(default)]
    pub is_encrypted: bool,
    #[serde(default)]
    pub is_blocked: bool,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub avatar_blob_ref: Option<BlobRef>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default, rename = "default_join_rule")]
    pub join_rule: Option<JoinRule>,
    #[serde(default)]
    pub history_visibility: Option<HistoryVisibility>,
    /// AKP-0007 (P3A.6) — `principal_control` vs `collaboration`.
    #[serde(default)]
    pub realm_class: Option<String>,
}

impl AdminRealm {
    pub fn discoverability_label(&self) -> Option<String> {
        self.discoverability.as_ref().map(wire_label)
    }

    pub fn join_rule_label(&self) -> Option<String> {
        self.join_rule.as_ref().map(wire_label)
    }

    pub fn type_label(&self) -> String {
        self.realm_class
            .as_deref()
            .unwrap_or("collaboration")
            .to_owned()
    }
}

fn wire_label<T>(value: &T) -> String
where
    T: Serialize,
{
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "-".to_owned())
}

/// MID-3 — read-only row in the per-Realm identity-audit diagnostic
/// page. One row per actor; lists the current effective
/// `ak.member.identity.update` event ids + the projection digest the
/// SPA computed. Used to triage cross-actor identity drift without
/// hitting the soland audit log directly.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RealmIdentityAuditRow {
    #[serde(default)]
    pub actor_id: String,
    /// R3.2 (UI-SOD-3) — SPA-derived primary handle. NOT a wire field:
    /// the operator view runs §3.2.1 selection over `handle_claims` to
    /// fill this (`crate::utils::security::primary_handle::select_primary_handle`).
    /// `None` means no verified candidate / selection not yet run.
    #[serde(skip)]
    pub primary_handle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// R3.2 (UI-SOD-5) — the handle captured when this identity row was
    /// last projected (`handle_at_time`, audit metadata only). When it
    /// differs from the §3.2.1-derived current primary handle the
    /// operator view surfaces a "handle changed since" hint. NOT an
    /// authoritative attribution field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle_at_time: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identity_event_ids: Vec<String>,
    /// R3.2 (UI-SOD-2) — renamed from `identity_state_digest`. Roster
    /// display-selection digest (member_roster_entry shape).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_display_state_digest: Option<String>,
    /// R3.2 (UI-SOD-2) — disclosed principal/holder DID. Gates the
    /// handle-claim fields below (dependentRequired).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,
    /// R3.2 (UI-SOD-2) — canonical `claim_digest` set of visible claims.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle_claim_digests: Option<Vec<String>>,
    /// R3.2 (UI-SOD-2) — inlined signed handle-claim evidence; the
    /// operator diag page runs primary-handle selection over this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle_claims: Option<Vec<HandleClaim>>,
    /// R3.2 (UI-SOD-2) — `true` when `handle_claims` is truncated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle_claims_limited: Option<bool>,
    /// `true` when the cached projection digest disagrees with the
    /// roster row — surface as a red flag in the operator view.
    #[serde(default)]
    pub cache_drift: bool,
}
