//! DTO shapes for the Realm admin surface.

use serde::{Deserialize, Serialize};

use crate::types::{HandleClaim, MemberDeliveryBinding};

// ── Realm types (security boundary) ──
//
// The object that carries encryption / join-rule / history-visibility /
// realm-class boundary fields is a Realm. Space containers are represented
// separately by `spaces::SpaceRow`.

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Realm {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub discoverability: Option<String>,
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
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default, rename = "default_join_rule")]
    pub join_rule: Option<String>,
    #[serde(default)]
    pub history_visibility: Option<String>,
    /// CKP-0007 (P3A.6) — `principal_control` vs `collaboration`.
    #[serde(default)]
    pub realm_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateRealmRequest {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub discoverability: Option<String>,
    #[serde(default, rename = "default_join_rule")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_rule: Option<String>,
    #[serde(default)]
    pub is_encrypted: bool,
    /// CKP-0007 (P3A.6) — required at create time; the spec pins this
    /// to `principal_control` / `collaboration`. Non-optional so the
    /// choice is always explicit on the wire (the admin UI's picker
    /// defaults it to `collaboration`); there is no silent reliance on
    /// the soland server-side default.
    pub realm_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealmMember {
    #[serde(default)]
    pub actor_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub joined_at: Option<String>,
    /// Spec 0a5ab85 — `ck.member.state{join}.delivery_status`. Drives
    /// the admin UI "routable" / "unroutable" indicator.
    #[serde(default)]
    pub delivery_status: Option<String>,
    /// Per-Realm delivery binding for this member. When present the
    /// admin UI MUST surface member_delivery_binding.recipient_service_did, binding_source,
    /// expiry, and the rebind action.
    #[serde(default)]
    pub delivery_binding: Option<MemberDeliveryBinding>,
    /// R3.1 (ROST-1) — membership state from the
    /// `member_roster_entry` wire shape (`join` / `invite` / `knock`).
    #[serde(default)]
    pub membership: Option<String>,
    /// R3.1 (ROST-1) — effective `ck.member.identity.update` event
    /// ids. Admin rows MUST source `display_name` / `primary_handle`
    /// from the joined effective MemberIdentity when this list is
    /// non-empty (MID-1).
    #[serde(default)]
    pub identity_event_ids: Vec<String>,
    /// R3.2 (ROST-1, UI-SOD-2) — roster display-selection digest. Renamed
    /// from the R3.1 `identity_state_digest`; the new computation folds in
    /// the visible handle-claim set (`handle / binding_state /
    /// expires_at`) on top of the effective identity event refs. Drives
    /// the "identity pending decryption" placeholder when the local cache
    /// disagrees. Wire shape: `member_roster_entry.member_display_state_digest`.
    #[serde(default)]
    pub member_display_state_digest: Option<String>,
    /// R3.2 (ROST-1) — disclosed principal/holder DID. This is the
    /// disclosure gate for the four handle-claim fields below: when the
    /// server has NOT disclosed `subject_id`, all of
    /// `handle_claim_digests` / `handle_claims` / `handle_claims_limited`
    /// MUST be absent too (dependentRequired). A missing claim set does
    /// NOT mean the subject has no handle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<String>,
    /// R3.2 (ROST-1) — canonical `claim_digest` set of the currently
    /// visible effective handle claims. Disclosure-gated on `subject_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle_claim_digests: Option<Vec<String>>,
    /// R3.2 (ROST-1) — inlined full signed `ck.schema.handle_claim.v1`
    /// evidence. Each claim's `subject` MUST equal this entry's
    /// `subject_id`. Disclosure-gated on `subject_id`. The SPA runs
    /// §3.2.1 primary-handle selection over this set to derive the
    /// display handle (see [`crate::utils::security::primary_handle`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle_claims: Option<Vec<HandleClaim>>,
    /// R3.2 (ROST-1) — `true` when `handle_claims` was truncated / only
    /// carries digest hints. Disclosure-gated on `subject_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle_claims_limited: Option<bool>,
    /// SPA-local derived display handle. R3.2: this is NOT a wire field —
    /// the roster MUST NOT carry a handle string directly. The SPA
    /// populates it by running §3.2.1 selection over `handle_claims`
    /// (`crate::utils::security::primary_handle::select_primary_handle`). `None`
    /// means selection has not run / no verified candidate yet.
    /// TODO(R3.2.1): wire the selection pass at projection-join time.
    #[serde(skip)]
    pub primary_handle: Option<String>,
}

/// MID-3 — read-only row in the per-Realm identity-audit diagnostic
/// page. One row per actor; lists the current effective
/// `ck.member.identity.update` event ids + the projection digest the
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
