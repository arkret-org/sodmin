pub use cokret_contracts::ops::HardeningStatus;
pub use cokret_core::model::HandleBindingState;
use serde::{Deserialize, Serialize};

// ── Pagination ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub total: u64,
    /// Opaque cursor for the next page. `None` when the current page is
    /// the last one.
    #[serde(default)]
    pub next_cursor: Option<String>,
}

// ── Actor types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Actor {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub did: String,
    #[serde(default)]
    pub handle: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub is_suspended: bool,
    #[serde(default)]
    pub is_deactivated: bool,
    /// Round 4 — `true` when the local 7-domain fanout completed but at
    /// least one federated peer has NOT yet confirmed the deactivation.
    /// The admin UI MUST NOT silently treat this principal as fully
    /// deactivated — surface the incomplete state explicitly.
    #[serde(default)]
    pub deactivation_federation_incomplete: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_active_at: Option<String>,
    #[serde(default)]
    pub device_count: u64,
    #[serde(default)]
    pub realm_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateActorRequest {
    #[serde(default)]
    pub handle: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateActorRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_suspended: Option<bool>,
}

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
    /// the visible handle-claim set (`claim_digest / binding_state /
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

/// `member_delivery_binding` shape carried inside [`RealmMember`].
/// Mirrors `event-payload.schema.json#/$defs/member_delivery_binding`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemberDeliveryBinding {
    #[serde(default)]
    pub recipient_service_did: String,
    #[serde(default)]
    pub binding_source: String,
    #[serde(default)]
    pub delivery_modes: Vec<String>,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub service_endpoint: Option<String>,
    /// `ck:event:*` ref to the service_acceptance event the recipient
    /// service signed for this binding (`explicit` / `invite` /
    /// `organization_policy` sources).
    #[serde(default)]
    pub service_acceptance_ref: Option<String>,
    /// Reference to the policy event that authorised this binding
    /// (`join_policy` / `organization_policy` / `realm_policy` sources).
    #[serde(default)]
    pub policy_event_ref: Option<String>,
}

// ── Device types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Device {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub device_type: Option<String>,
    #[serde(default)]
    pub last_seen_ip: Option<String>,
    #[serde(default)]
    pub last_seen_ts: Option<u64>,
    #[serde(default)]
    pub verification_status: Option<String>,
    #[serde(default)]
    pub is_cross_signed: bool,
}

// ── Capability types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityGrant {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub grantor_id: String,
    #[serde(default)]
    pub grantee_id: String,
    #[serde(default)]
    pub capability: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub constraints: Option<serde_json::Value>,
    #[serde(default)]
    pub granted_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub is_revoked: bool,
    #[serde(default)]
    pub delegation_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GrantCapabilityRequest {
    pub grantee_id: String,
    pub capability: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// PATCH body for `/_soland/admin/capabilities/{id}` — fine-grained
/// edits to an existing grant's constraints. All fields optional; the
/// admin only sends the keys that actually changed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCapabilityRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_write_fields: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets_allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_required: Option<bool>,
}

// ── Federation types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FederationPeer {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub trust_level: Option<String>,
    #[serde(default)]
    pub last_successful_txn: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub retry_interval: u64,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FederationAllowRule {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub rule_type: Option<String>,
    #[serde(default)]
    pub polarity: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub allowlist_enforced: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddFederationRuleRequest {
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polarity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowlist_enforced: Option<bool>,
}

// ── Applet types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Applet {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub applet_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub endpoint_url: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub registered_at: Option<String>,
    #[serde(default)]
    pub last_transaction_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegisterAppletRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applet_type: Option<String>,
    pub endpoint_url: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

// ── Agent types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Agent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub owner_id: String,
    #[serde(default)]
    pub agent_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_active_at: Option<String>,
    /// CKP-0008 — reducer-stamped actor kind. Native personal agents
    /// use `agent`; Applet-managed ghost actors use `integration` or
    /// `agent` plus provenance/accountability metadata.
    /// Populated by soland's `ck.self.agent.list` / `ck.self.agent.get`.
    #[serde(default)]
    pub actor_kind: Option<String>,
    /// CKP-0008 — controller DID. Native personal agents are 1:1 bound
    /// to a controller DID; Applet-managed actors point at their owning
    /// applet or integration provenance instead.
    #[serde(default)]
    pub controller_did: Option<String>,
    /// CKP-0008 — current `accountability_grant` id (coauth-issued).
    /// `None` when no grant has been issued / the existing one was
    /// revoked.
    #[serde(default)]
    pub accountability_grant_id: Option<String>,
    /// CKP-0008 — ISO-8601 timestamp of when the
    /// `accountability_grant` was last refreshed. Drives the
    /// "accountability grant freshness" indicator on the detail page.
    #[serde(default)]
    pub accountability_grant_refreshed_at: Option<String>,
    /// CKP-0008 — current pairing status (e.g. `paired`, `pending`,
    /// `expired`). Surfaced verbatim on the detail page.
    #[serde(default)]
    pub pairing_status: Option<String>,
    /// CKP-0008 — list of authorized agent key DIDs.
    #[serde(default)]
    pub agent_keys: Vec<String>,
}

/// CKP-0008 — capability grant detail for the personal-agent detail
/// view's grant editor. Mirrors the soland `agent.grant.attach` /
/// `agent.grant.detach` payload.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentGrantEntry {
    #[serde(default)]
    pub grant_id: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub issued_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// CKP-0008 — agent provision wizard request body.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentProvisionRequest {
    #[serde(default)]
    pub controller_did: String,
    #[serde(default)]
    pub display_name: Option<String>,
    /// Agent key proof material from step 2 of the wizard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_key_proof: Option<serde_json::Value>,
}

/// CKP-0008 — agent provision wizard response (returns the freshly
/// issued agent principal DID).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentProvisionResponse {
    #[serde(default)]
    pub agent_principal_id: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub initial_grant_ids: Vec<String>,
}

/// CKP-0008 — coauth `accountability_grant` request body for the
/// wizard's controller-approval step.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountabilityGrantRequest {
    #[serde(default)]
    pub controller_did: String,
    /// Optional human-friendly rationale stored on the grant ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountabilityGrantResponse {
    #[serde(default)]
    pub accountability_grant_id: String,
    #[serde(default)]
    pub issued_at: Option<String>,
}

// ── Key-backup admin types (B-C) ──

/// B-C — recovery policy lifecycle. Mirrors
/// `ck.schema.recovery_policy.v1#lifecycle`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecoveryPolicy {
    #[serde(default)]
    pub policy_id: String,
    /// `pending` / `active` / `retired`.
    #[serde(default)]
    pub lifecycle: String,
    #[serde(default)]
    pub kdf_profile: Option<String>,
    #[serde(default)]
    pub epoch_hash: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyBackupSeries {
    #[serde(default)]
    pub series_id: String,
    #[serde(default)]
    pub backup_class: Option<String>,
    /// Highest `series_seq` observed for this series.
    #[serde(default)]
    pub frontier_seq: u64,
    #[serde(default)]
    pub frontier_ref: Option<String>,
    /// Spec rename (head 37ce729): `series_sequence` → `series_seq`.
    #[serde(default)]
    pub series_seq: u64,
    /// Three-class 409 reason counters per §12.1.
    #[serde(default)]
    pub series_chain_broken_count: u64,
    #[serde(default)]
    pub series_seq_not_monotonic_count: u64,
    #[serde(default)]
    pub series_predecessor_not_found_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecoveryReceipt {
    #[serde(default)]
    pub receipt_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub policy_id: Option<String>,
    #[serde(default)]
    pub issued_at: Option<String>,
    #[serde(default)]
    pub verified: bool,
}

// ── Report / Moderation types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Report {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub realm_id: Option<String>,
    #[serde(default)]
    pub event_id: Option<String>,
    #[serde(default)]
    pub reporter_id: Option<String>,
    #[serde(default)]
    pub reported_actor_id: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub score: Option<i64>,
    #[serde(default)]
    pub event_content: Option<serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub resolved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateReportRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

// ── Invite token types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InviteToken {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub uses_allowed: Option<u64>,
    #[serde(default)]
    pub uses_completed: u64,
    #[serde(default)]
    pub uses_pending: u64,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub realm_id: Option<String>,
    // Audit pair: `created_by` precedes `created_at`, matching the
    // project-wide (and canonical) ordering convention.
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateInviteTokenRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses_allowed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm_id: Option<String>,
}

// ── Audit types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub target_type: Option<String>,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub details: Option<serde_json::Value>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    /// CKP-0007 — the effective scope at which the action took effect
    /// (`ck:realm:...` or `ck:circle:...`). Distinct from the audited
    /// `target_id` because Circle actions surface inside a Realm
    /// envelope but get pinned to the Circle for replay-locality.
    /// `None` for legacy entries written before the field shipped.
    #[serde(default)]
    pub effective_scope: Option<String>,
    /// CKP-0007 — when `effective_scope` points at a Circle, this is
    /// the parent realm id so the audit row can render a "jump to
    /// Realm" link without an extra round trip.
    #[serde(default)]
    pub scope_realm_id: Option<String>,
    /// CKP-0007 — convenience copy of `effective_scope` when it is a
    /// `ck:circle:...` id; saves the row a string-prefix sniff on
    /// the rendering path.
    #[serde(default)]
    pub scope_circle_id: Option<String>,
    /// CKP-0008 — when the envelope was signed/executed on behalf of
    /// the principal, this records the executing DID (e.g. a personal
    /// agent acting on behalf of the controller). Conditional: present
    /// only on agent-attributed envelopes.
    #[serde(default)]
    pub executed_by: Option<String>,
    /// CKP-0008 — typed id of the `accountability_grant` or capability
    /// grant whose validity authorized the action. Lets the audit row
    /// link back to the grant ledger row.
    #[serde(default)]
    pub authorization_ref: Option<String>,
    /// CKP-0008 — reducer-stamped projection of the actor classification
    /// at the moment of admission. One of `user` / `org` / `team` /
    /// `agent` / `service` / `device` / `integration`. Immutable per
    /// envelope and supplied by the reducer; clients MUST NOT attempt
    /// to set this on write.
    #[serde(default)]
    pub actor_kind: Option<String>,
}

impl AuditEntry {
    /// Classify the audit entry's effective scope for badge / link
    /// rendering. Pure helper so the rule stays unit-testable.
    pub fn scope_kind(&self) -> AuditScopeKind {
        if let Some(ref s) = self.scope_circle_id
            && !s.is_empty()
        {
            return AuditScopeKind::Circle(s.clone());
        }
        if let Some(ref s) = self.effective_scope {
            if s.starts_with("ck:circle:") {
                return AuditScopeKind::Circle(s.clone());
            }
            if s.starts_with("ck:realm:") {
                return AuditScopeKind::Realm(s.clone());
            }
        }
        if let Some(ref r) = self.scope_realm_id
            && !r.is_empty()
        {
            return AuditScopeKind::Realm(r.clone());
        }
        AuditScopeKind::Unknown
    }
}

/// Discriminated effective-scope value for the audit views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditScopeKind {
    Realm(String),
    Circle(String),
    Unknown,
}

// ── Policy types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Policy {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub policy_type: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub rules: Option<serde_json::Value>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreatePolicyRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<serde_json::Value>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub priority: i32,
}

// ── Media / Blob types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaStatistics {
    #[serde(default)]
    pub total_blobs: u64,
    #[serde(default)]
    pub total_size: u64,
    #[serde(default)]
    pub quarantined_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActorMediaStatistics {
    #[serde(default)]
    pub actor_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub blob_count: u64,
    #[serde(default)]
    pub total_size: u64,
}

// ── Server info types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerInfo {
    #[serde(default)]
    pub server_version: String,
    #[serde(default)]
    pub protocol_version: Option<String>,
    #[serde(default)]
    pub server_name: Option<String>,
    #[serde(default)]
    pub uptime: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ClaimedProfile {
    Claim(ClaimedProfileClaim),
    ProfileId(String),
}

impl ClaimedProfile {
    pub fn profile_id(&self) -> &str {
        match self {
            Self::Claim(claim) => claim.profile_id.as_str(),
            Self::ProfileId(profile_id) => profile_id.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ClaimedProfileClaim {
    #[serde(default)]
    pub profile_id: String,
    #[serde(default)]
    pub claim_kind: Option<String>,
    #[serde(default)]
    pub claimed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum VerifiedProfile {
    Claim(VerifiedProfileClaim),
    ProfileId(String),
}

impl VerifiedProfile {
    pub fn profile_id(&self) -> &str {
        match self {
            Self::Claim(claim) => claim.profile_id.as_str(),
            Self::ProfileId(profile_id) => profile_id.as_str(),
        }
    }

    pub fn artifact_ref(&self) -> Option<&str> {
        match self {
            Self::Claim(claim) => claim.artifact_ref.as_deref(),
            Self::ProfileId(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct VerifiedProfileClaim {
    #[serde(default)]
    pub profile_id: String,
    #[serde(default)]
    pub claim_kind: Option<String>,
    #[serde(default)]
    pub cotest_run_id: Option<String>,
    #[serde(default)]
    pub artifact_digest: Option<String>,
    #[serde(default)]
    pub artifact_ref: Option<String>,
    #[serde(default)]
    pub cotest_issuer_did: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum CompatSurface {
    Surface(CompatSurfaceClaim),
    Name(String),
}

impl CompatSurface {
    pub fn name(&self) -> &str {
        match self {
            Self::Surface(surface) => surface.name.as_str(),
            Self::Name(name) => name.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CompatSurfaceClaim {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerDescribeResBody {
    #[serde(default)]
    pub service_did: String,
    /// Round 4 — required `trust_domain` per ServerDescribe v2 (spec
    /// a77b995). Anchors `ck.cross_signing.publish` proofs and federation
    /// canonical transcripts; mismatch is the wire-breaker.
    #[serde(default)]
    pub trust_domain: Option<String>,
    #[serde(default)]
    pub service_type: Option<String>,
    #[serde(default)]
    pub protocol_version: Option<String>,
    #[serde(default)]
    pub supported_profiles: Vec<String>,
    #[serde(default)]
    pub supported_features: Vec<String>,
    #[serde(default)]
    pub supported_operations: Vec<String>,
    #[serde(default)]
    pub supported_reducer_profiles: Vec<String>,
    #[serde(default)]
    pub supported_schema_profiles: Vec<String>,
    #[serde(default)]
    pub supported_bindings: Vec<serde_json::Value>,
    #[serde(default)]
    pub auth_metadata: Option<ServerDescribeAuthMetadata>,
    #[serde(default)]
    pub identity_registry_resolver: Option<IdentityRegistryResolverDescriptor>,
    #[serde(default)]
    pub admin_audience: Option<String>,
    #[serde(default)]
    pub openapi_version: Option<String>,
    #[serde(default)]
    pub schema_registry_version: Option<String>,
    #[serde(default)]
    pub event_kind_registry_version: Option<String>,
    #[serde(default)]
    pub registry: serde_json::Value,
    #[serde(default)]
    pub limits: serde_json::Value,
    /// Round 4 — `rate_limit` oneOf (window / token-bucket / disabled).
    /// Free-form JSON because the SDK still exposes it as `Value`.
    #[serde(default)]
    pub rate_limit: serde_json::Value,
    /// T1.4 — soland surfaces its dev-mode posture directly on
    /// `/_cokret/describe` (and `/health`). Sodmin uses this to
    /// render the red top-of-page banner. `None` for older servers that
    /// predate the field.
    #[serde(default)]
    pub development_mode: Option<bool>,
    /// T1.4 — `"development"` | `"production"`. Mirrors
    /// [`Self::development_mode`]; when `development_mode == true`,
    /// soland accepts unsigned / weakly-signed envelopes.
    #[serde(default)]
    pub proof_verifier_mode: Option<String>,
    /// T1.4 — effective admin-API auth posture:
    /// `"development"` | `"did_allowlist"` | `"oauth_introspection"` | `"closed"`.
    #[serde(default)]
    pub admin_auth_mode: Option<String>,
    /// Round 4 — concrete `implemented_features` list (subset of
    /// supported_features that this build actually wires up). Distinct
    /// from `supported_features` which advertises the capability surface.
    #[serde(default)]
    pub implemented_features: Vec<String>,
    /// T6.1 — profiles the server has been independently verified to
    /// implement against the spec test suite. Rendered as green
    /// "verified" chips.
    #[serde(default)]
    pub verified_profiles: Vec<VerifiedProfile>,
    /// T6.1 — profiles the operator self-claims support for. Rendered
    /// as yellow "self-claimed" chips because they lack third-party
    /// verification.
    #[serde(default)]
    pub claimed_profiles: Vec<ClaimedProfile>,
    /// T6.1 — experimental features the server exposes. Rendered as
    /// blue chips with an "unstable" warning.
    #[serde(default)]
    pub experimental_features: Vec<String>,
    /// T6.1 — compatibility surfaces (legacy / shim endpoints). Grey
    /// chips.
    #[serde(default)]
    pub compat_surfaces: Vec<CompatSurface>,
    /// Round 4 — `plaintext_visibility` snapshot (services whose plaintext
    /// bodies remain readable on this deployment). Renamed from the
    /// pre-round-4 `plaintext_visible_services`; the legacy field name is
    /// still accepted on the wire via `alias` for in-flight upgrades.
    #[serde(default, alias = "plaintext_visible_services")]
    pub plaintext_visibility: Vec<String>,
    /// T8.3 — production hardening checklist snapshot, mirrored from
    /// `/health`. Older servers that predate the field omit it.
    #[serde(default)]
    pub hardening: Option<HardeningStatus>,
}

impl ServerDescribeResBody {
    /// Round 4 — render-time check used by the ServerDescribe v2 admin
    /// view. When `development_mode == true` AND `verified_profiles` is
    /// non-empty the server is making contradictory claims (relaxed
    /// proof verifier breaks the verification chain). The UI surfaces a
    /// red warning banner in this case.
    pub fn dev_mode_with_verified_profiles(&self) -> bool {
        self.development_mode.unwrap_or(false) && !self.verified_profiles.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerDescribeAuthMetadata {
    #[serde(default)]
    pub issuer_did: Option<String>,
    #[serde(default)]
    pub oauth_issuer: Option<String>,
    #[serde(default)]
    pub openid_configuration: Option<String>,
    #[serde(default)]
    pub required_audience: Option<String>,
    #[serde(default)]
    pub admin_audience: Option<String>,
    #[serde(default)]
    pub supported_auth_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityRegistryResolverDescriptor {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub delegated_resolver: Option<IdentityRegistryDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityRegistryDescriptor {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub resolver: Option<String>,
    #[serde(default)]
    pub proof_required_for_pairwise: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerStats {
    #[serde(default)]
    pub actor_count: u64,
    #[serde(default)]
    pub active_actor_count: u64,
    #[serde(default)]
    pub realm_count: u64,
    #[serde(default)]
    pub report_count: u64,
    #[serde(default)]
    pub federation_peer_count: u64,
    #[serde(default)]
    pub applet_count: u64,
    #[serde(default)]
    pub agent_count: u64,
    #[serde(default)]
    pub blob_count: u64,
    #[serde(default)]
    pub blob_total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerStatusComponent {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerStatusResponse {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub results: Vec<ServerStatusComponent>,
}

// ── Handle availability ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandleAvailabilityResult {
    #[serde(default)]
    pub available: bool,
    #[serde(default)]
    pub error: Option<String>,
}

// ── Handle claim evidence (R3.2 — ck.schema.handle_claim.v1) ──

/// R3.2 — local mirror of the signed `ck.schema.handle_claim.v1` object
/// the wire now carries inline inside roster entries
/// (`member_roster_entry.handle_claims[]`) and the
/// `ck.find.directory.list_handles_for_subject` response. Handle lifecycle has
/// fully moved off `MemberIdentity` onto this claim object (cokret-spec
/// @ b56cab1). Mirrors the SDK `cokret_core::model::handle::HandleClaim`;
/// only the fields the admin UI renders / runs selection over are kept.
///
/// Note: `claim_kind=service_handle` is REMOVED in v1 — the enum only
/// accepts `handle_binding` / `organization_handle`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct HandleClaim {
    /// Canonical handle string (`localpart:domain`). Audit/display only;
    /// the authoritative subject is `subject` (a holder/principal DID).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handle_aliases: Vec<String>,
    /// Holder/principal DID. MUST equal the enclosing roster entry /
    /// response `subject_id` (byte-equal); mismatch is dropped/fails closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Issuer DID that signed this claim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_state: Option<HandleBindingState>,
    /// Audience scope (e.g. `ck:realm:*`). When present, only matches a
    /// resolution context equal to this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
    /// ISO-8601 issuance timestamp. Canonical wire name is `created_at`
    /// (`ck.schema.handle_claim.v1`); the §3.2.1 primary-handle
    /// selection tie-break orders on this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// ISO-8601 expiry timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Canonical `claim_digest` hint, when the server pre-computed it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_digest: Option<String>,
}

/// R3.2 (UI-SOD-4) — request body for `ck.find.directory.list_handles_for_subject`
/// (`POST /_cokret/find/directory/list-handles-for-subject`). Known
/// holder/principal DID → currently visible signed handle claims, the
/// inverse of `resolve_handle`. Mirrors the SDK
/// `DirectoryListHandlesForSubjectReqBody`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListHandlesForSubjectRequest {
    /// Holder/principal DID reverse-lookup key. MUST NOT be a Realm
    /// `actor_id` / `account_id` / service DID.
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requester: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// R3.2 (UI-SOD-4) — response body for
/// `ck.find.directory.list_handles_for_subject`. Schema
/// `ck.schema.list_handles_for_subject_response.v1`. Every
/// `claims[].subject` MUST equal [`Self::subject`] (byte-equal);
/// mismatching claims MUST be dropped or the response failed closed —
/// see [`Self::visible_claims`]. Mirrors the SDK
/// `DirectoryListHandlesForSubjectResBody`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListHandlesForSubjectResponse {
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub claims: Vec<HandleClaim>,
    /// Server-side §3.2.1 primary-handle selection result, when the
    /// directory ran it. The SPA MAY re-derive locally via
    /// [`crate::utils::security::primary_handle::select_primary_handle`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_handle: Option<String>,
    #[serde(default)]
    pub as_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

impl ListHandlesForSubjectResponse {
    /// Fail-closed view of `claims`: drops any claim whose `subject` is
    /// not byte-equal to the response `subject` (schema invariant of
    /// `ck.schema.list_handles_for_subject_response.v1`).
    pub fn visible_claims(&self) -> Vec<&HandleClaim> {
        self.claims
            .iter()
            .filter(|c| c.subject.as_deref() == Some(self.subject.as_str()))
            .collect()
    }
}

// ── Handle management (T6.2 §2) ──

/// One row in `GET /_soland/admin/handles`. Mirrors the `ck.handle.*` cell
/// projection — `canonical_uri` is the cell subject, `aliases` is the
/// projected handle set, `issuer_did` is the principal that signed the
/// most recent assignment Move.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandleRecord {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub canonical_uri: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub issuer_did: Option<String>,
    #[serde(default)]
    pub subject_id: Option<String>,
    #[serde(default)]
    pub assigned_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub last_reassignment_at: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Single audit event for `GET /_soland/admin/handles/{id}/audit`. The
/// audit table is what T3.2 created — we surface the minimum the
/// operator needs to triage.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandleAuditEvent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub previous_subject_id: Option<String>,
    #[serde(default)]
    pub new_subject_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandleReassignRequest {
    pub new_subject_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// ── Delivery binding policy (T6.2 §3) ──

/// Effective `ck.cell.realm.delivery_binding_policy` for a Realm.
/// `allowed_recipient_services` and `binding_source_policy` are
/// operator-mutable; `policy_frontier` is written by the soland
/// reducer and is therefore read-only on the admin surface.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealmDeliveryBindingPolicy {
    /// Realm identifier (security boundary).
    #[serde(default)]
    pub realm_id: String,
    #[serde(default)]
    pub allowed_recipient_services: Vec<String>,
    #[serde(default)]
    pub binding_source_policy: Option<String>,
    #[serde(default)]
    pub policy_frontier: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateDeliveryBindingPolicyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_recipient_services: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_source_policy: Option<String>,
}

/// One row in the per-Realm "is each member routable?" check table.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemberRoutabilityRow {
    #[serde(default)]
    pub actor_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub recipient_service_did: Option<String>,
    #[serde(default)]
    pub in_allowed_list: bool,
    #[serde(default)]
    pub delivery_status: Option<String>,
}

// ── Round 4 — Delivery binding handover (error codes
//     delivery_binding_stale / delivery_binding_handed_over /
//     historical_only) ──────────────────────────────────────────────

/// Round 4 — discriminated reason a delivery-binding handover row
/// surfaces. The first two are wire-breaking failures the operator must
/// act on; `HistoricalOnly` is a 200 diagnostic that documents a
/// cached-replay response and MUST NOT be presented as a fresh action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryBindingHandoverReason {
    /// `delivery_binding_stale` (HTTP 409). Recipient rejected the
    /// envelope because its binding has moved on. Retry against
    /// `new_recipient_service_did` at/after `handover_frontier`.
    DeliveryBindingStale,
    /// `delivery_binding_handed_over` (HTTP 409). Recipient has
    /// permanently handed delivery off; submissions MUST switch to
    /// `new_recipient_service_did`.
    DeliveryBindingHandedOver,
    /// `historical_only` (HTTP 200, diagnostic). Cached replay against a
    /// prior key state. Information only — NOT a fresh action.
    HistoricalOnly,
}

/// Round 4 — one row in the delivery-binding handover panel. Surfaces
/// the new error-code triple plus the redirect target + frontier the
/// handover advertises.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DeliveryBindingHandoverRow {
    #[serde(default)]
    pub realm_id: String,
    #[serde(default)]
    pub actor_id: String,
    /// The previous (now-stale) recipient service DID, if known.
    #[serde(default)]
    pub previous_recipient_service_did: Option<String>,
    /// `new_recipient_service_did` — the redirect target the recipient
    /// service advertises in the `delivery_binding_stale` /
    /// `delivery_binding_handed_over` 409 body.
    #[serde(default)]
    pub new_recipient_service_did: Option<String>,
    /// `handover_frontier` — the frontier (vector of `ck:event:*` refs)
    /// at/after which the new recipient takes effect.
    #[serde(default)]
    pub handover_frontier: Vec<String>,
    #[serde(default)]
    pub reason_code: Option<String>,
    #[serde(default)]
    pub observed_at: Option<String>,
}

impl DeliveryBindingHandoverRow {
    pub fn classified_reason(&self) -> Option<DeliveryBindingHandoverReason> {
        match self.reason_code.as_deref() {
            Some("delivery_binding_stale") => {
                Some(DeliveryBindingHandoverReason::DeliveryBindingStale)
            }
            Some("delivery_binding_handed_over") => {
                Some(DeliveryBindingHandoverReason::DeliveryBindingHandedOver)
            }
            Some("historical_only") => Some(DeliveryBindingHandoverReason::HistoricalOnly),
            _ => None,
        }
    }
}

// ── Round 4 — 3PID invite admin row ─────────────────────────────────

/// Round 4 — terminal state for an admin-visible 3PID invite. Every
/// terminal value MUST be displayed truthfully; in particular
/// `send_failed` is a permanent failure (the OOB code was never
/// delivered) and the admin UI must not paper it over as success.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThirdPartyInviteTerminalState {
    Claimed,
    SendFailed,
    RevokedByCapabilityLoss,
    RevokedByInviterLeft,
    InvalidatedByRateLimit,
}

/// Round 4 — admin-visible 3PID invite row. The plaintext 3PID is
/// intentionally absent — the wire never carries it, and the admin UI
/// never reconstructs it. `evidence` is opaque (commitment digest /
/// lookup ref / pepper id) and stays untrusted on the client.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ThirdPartyInviteRow {
    #[serde(default)]
    pub invite_id: String,
    #[serde(default)]
    pub oob_code_kind: Option<String>,
    #[serde(default)]
    pub verification_service_did: Option<String>,
    /// Slug of the terminal state. Use [`Self::classified_state`] for
    /// the strongly-typed enum.
    #[serde(default)]
    pub terminal_state: Option<String>,
    /// Opaque inspection helper: token-commitment / lookup_table_ref /
    /// pepper_id (whichever applies). Never the plaintext.
    #[serde(default)]
    pub evidence: Option<String>,
    #[serde(default)]
    pub observed_at: Option<String>,
}

impl ThirdPartyInviteRow {
    pub fn classified_state(&self) -> Option<ThirdPartyInviteTerminalState> {
        match self.terminal_state.as_deref() {
            Some("claimed") => Some(ThirdPartyInviteTerminalState::Claimed),
            Some("send_failed") => Some(ThirdPartyInviteTerminalState::SendFailed),
            Some("revoked_by_capability_loss") => {
                Some(ThirdPartyInviteTerminalState::RevokedByCapabilityLoss)
            }
            Some("revoked_by_inviter_left") => {
                Some(ThirdPartyInviteTerminalState::RevokedByInviterLeft)
            }
            Some("invalidated_by_rate_limit") => {
                Some(ThirdPartyInviteTerminalState::InvalidatedByRateLimit)
            }
            _ => None,
        }
    }
}

// ── Realm link graph (R5.2, Round R1.2 — ck.realm.link projection) ──

/// One outbound / inbound typed link between two Realm boundaries.
/// Backed by `ck.realm.link` reducer_input events. Common `link_kind`
/// values include `governed_by`, `discoverable_from`, `mirror_of`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealmLinkRow {
    /// Source Realm id (the "from" boundary).
    #[serde(default)]
    pub source_realm_id: String,
    /// Target Realm id (the "to" boundary).
    #[serde(default)]
    pub target_realm_id: String,
    /// Typed link kind. Free-form string at the wire layer; the UI
    /// recognises a small set and renders the rest as `other`.
    #[serde(default)]
    pub link_kind: String,
    /// Human-friendly label for the target Realm, if soland resolves it.
    #[serde(default)]
    pub target_display_name: Option<String>,
    /// HLC / cell timestamp of the last event that established or
    /// refreshed this link.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Whether this row appears in the inbound list (vs outbound).
    /// Surface-level convenience; the server fills it when the response
    /// covers both directions in a single payload.
    #[serde(default)]
    pub inbound: bool,
}

// ── Push route / device route (T6.2 §4) ──

/// Surface for `ck.device.push_route` cells, grouped by principal so
/// the operator can inspect what each user is currently subscribed to
/// without leaking the raw `push_target_id`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PushRouteRow {
    #[serde(default)]
    pub principal_id: String,
    #[serde(default)]
    pub device_id: String,
    #[serde(default)]
    pub cell_subject: String,
    /// The 4-tuple `(principal_id, device_id, transport, route_id)`
    /// formatted for human display. Soland already emits this.
    #[serde(default)]
    pub cell_subject_tuple: Vec<String>,
    #[serde(default)]
    pub transport: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    /// Opaque, sensitive. The UI must keep this collapsed by default.
    #[serde(default)]
    pub push_target_id: Option<String>,
    #[serde(default)]
    pub last_rotation_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        HandleRecord, ServerDescribeResBody, UpdateCapabilityRequest,
        UpdateDeliveryBindingPolicyRequest,
    };

    #[test]
    fn server_describe_accepts_principal_server_profile_status() {
        let describe: ServerDescribeResBody = serde_json::from_value(json!({
            "service_did": "did:web:soland.local",
            "service_type": "principal_server",
            "protocol_version": "1.0",
            "supported_profiles": ["ck.profile.principal_server.v1"],
            "supported_features": ["events.describe", "events.submit"],
            "supported_operations": ["ck.self.events.submit"],
            "supported_reducer_profiles": ["ck.reducer.v1"],
            "supported_schema_profiles": ["ck.schema.core.v1"],
            "limits": {
                "profile_status": {
                    "conformance": "limited_reference",
                    "implemented_surfaces": ["principal_server", "events_api_minimal"]
                }
            }
        }))
        .expect("soland server describe should deserialize");

        assert_eq!(describe.service_did, "did:web:soland.local");
        assert_eq!(describe.service_type.as_deref(), Some("principal_server"));
        assert_eq!(describe.protocol_version.as_deref(), Some("1.0"));
        assert_eq!(
            describe.supported_reducer_profiles,
            vec!["ck.reducer.v1".to_string()]
        );
        assert_eq!(
            describe.limits["profile_status"]["conformance"],
            "limited_reference"
        );
    }

    #[test]
    fn server_describe_accepts_coauth_issuer_and_registry() {
        let describe: ServerDescribeResBody = serde_json::from_value(json!({
            "service_did": "did:web:auth.example.com",
            "service_type": "auth_account_server",
            "protocol_version": "1.0",
            "supported_profiles": ["ck.profile.auth_account.v1"],
            "auth_metadata": {
                "issuer_did": "did:web:auth.example.com#issuer",
                "oauth_issuer": "https://auth.example.com"
            },
            "identity_registry_resolver": {
                "mode": "delegated_resolver",
                "endpoint": "https://auth.example.com/_cokret/root/identity/resolve",
                "delegated_resolver": {
                    "kind": "public_did_resolver",
                    "resolver": "https://resolver.example.com/"
                }
            }
        }))
        .expect("coauth server describe should deserialize");

        assert_eq!(
            describe
                .auth_metadata
                .as_ref()
                .and_then(|metadata| metadata.issuer_did.as_deref()),
            Some("did:web:auth.example.com#issuer")
        );
        assert_eq!(
            describe
                .identity_registry_resolver
                .as_ref()
                .and_then(|registry| registry.delegated_resolver.as_ref())
                .and_then(|delegated| delegated.resolver.as_deref()),
            Some("https://resolver.example.com/")
        );
    }

    #[test]
    fn server_describe_reads_supported_profiles() {
        let describe: ServerDescribeResBody = serde_json::from_value(json!({
            "service_did": "did:web:identity.example",
            "supported_profiles": ["ck.profile.identity_registry.v1"]
        }))
        .expect("supported_profiles should deserialize");

        assert_eq!(
            describe.supported_profiles,
            vec!["ck.profile.identity_registry.v1".to_string()]
        );
    }

    #[test]
    fn server_describe_reads_conformance_buckets() {
        let describe: ServerDescribeResBody = serde_json::from_value(json!({
            "service_did": "did:web:soland.local",
            "verified_profiles": [{
                "profile_id": "ck.profile.principal_server.v1",
                "claim_kind": "cotest_verified",
                "artifact_ref": "ck:artifact:principal-server-run"
            }],
            "claimed_profiles": [{
                "profile_id": "ck.profile.identity_registry.v1",
                "claim_kind": "self_claimed",
                "claimed_at": "2026-06-05T00:00:00Z"
            }],
            "experimental_features": ["events.replay.v2"],
            "compat_surfaces": [{
                "name": "legacy.federation.v0",
                "kind": "legacy"
            }],
            "plaintext_visible_services": ["floria"],
        }))
        .expect("conformance buckets should deserialize");

        assert_eq!(
            describe.verified_profiles[0].profile_id(),
            "ck.profile.principal_server.v1"
        );
        assert_eq!(
            describe.verified_profiles[0].artifact_ref(),
            Some("ck:artifact:principal-server-run")
        );
        assert_eq!(
            describe.claimed_profiles[0].profile_id(),
            "ck.profile.identity_registry.v1"
        );
        assert_eq!(
            describe.experimental_features,
            vec!["events.replay.v2".to_string()]
        );
        assert_eq!(describe.compat_surfaces[0].name(), "legacy.federation.v0");
        assert_eq!(describe.plaintext_visibility, vec!["floria".to_string()]);
    }

    #[test]
    fn server_describe_keeps_legacy_string_conformance_buckets_compatible() {
        let describe: ServerDescribeResBody = serde_json::from_value(json!({
            "service_did": "did:web:soland.local",
            "verified_profiles": ["ck.profile.principal_server.v1"],
            "claimed_profiles": ["ck.profile.identity_registry.v1"],
            "compat_surfaces": ["legacy.federation.v0"],
        }))
        .expect("legacy conformance buckets should deserialize");

        assert_eq!(
            describe.verified_profiles[0].profile_id(),
            "ck.profile.principal_server.v1"
        );
        assert_eq!(
            describe.claimed_profiles[0].profile_id(),
            "ck.profile.identity_registry.v1"
        );
        assert_eq!(describe.compat_surfaces[0].name(), "legacy.federation.v0");
    }

    #[test]
    fn handle_record_round_trip() {
        let record: HandleRecord = serde_json::from_value(json!({
            "id": "h-1",
            "canonical_uri": "ck:handle:@alice",
            "aliases": ["@alice", "@alice.example"],
            "issuer_did": "did:web:auth.example.com",
            "subject_id": "did:key:zABC",
            "status": "active"
        }))
        .expect("handle record should deserialize");

        assert_eq!(record.canonical_uri, "ck:handle:@alice");
        assert_eq!(record.aliases.len(), 2);
        assert_eq!(record.status.as_deref(), Some("active"));
    }

    #[test]
    fn delivery_binding_policy_request_omits_none() {
        let req = UpdateDeliveryBindingPolicyRequest {
            allowed_recipient_services: Some(vec!["did:web:floria.example".to_string()]),
            binding_source_policy: None,
        };
        let serialized = serde_json::to_string(&req).expect("serializes");
        assert!(serialized.contains("allowed_recipient_services"));
        assert!(!serialized.contains("binding_source_policy"));
    }

    #[test]
    fn update_capability_request_serializes_only_set_fields() {
        let req = UpdateCapabilityRequest {
            expires_at: Some("2027-01-01T00:00:00Z".to_string()),
            allowed_write_fields: Some(vec!["body.text".to_string()]),
            facets_allow: None,
            approval_required: Some(true),
        };
        let serialized = serde_json::to_string(&req).expect("serializes");
        assert!(serialized.contains("expires_at"));
        assert!(serialized.contains("allowed_write_fields"));
        assert!(serialized.contains("approval_required"));
        assert!(!serialized.contains("facets_allow"));
    }
}
