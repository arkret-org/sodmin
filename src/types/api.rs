use serde::{Deserialize, Serialize};

// ── Pagination ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaginationParams {
    pub page: u64,
    pub per_page: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
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
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_active_at: Option<String>,
    #[serde(default)]
    pub device_count: u64,
    #[serde(default)]
    pub space_count: u64,
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

// ── Space types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Space {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub space_type: Option<String>,
    #[serde(default)]
    pub discoverability: Option<String>,
    #[serde(default)]
    pub creator_id: Option<String>,
    #[serde(default)]
    pub member_count: u64,
    #[serde(default)]
    pub is_encrypted: bool,
    #[serde(default)]
    pub is_blocked: bool,
    #[serde(default)]
    pub parent_space_id: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub join_rule: Option<String>,
    #[serde(default)]
    pub history_visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateSpaceRequest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub space_type: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub discoverability: Option<String>,
    #[serde(default)]
    pub join_rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_space_id: Option<String>,
    #[serde(default)]
    pub is_encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpaceMember {
    #[serde(default)]
    pub actor_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub joined_at: Option<String>,
    /// Spec 0a5ab85 — `cx.member.state{join}.delivery_status`. Drives
    /// the admin UI "routable" / "unroutable" indicator.
    #[serde(default)]
    pub delivery_status: Option<String>,
    /// Per-Space delivery binding for this member. When present the
    /// admin UI MUST surface recipient_service_did, binding_source,
    /// expiry, and the rebind action.
    #[serde(default)]
    pub delivery_binding: Option<MemberDeliveryBinding>,
}

/// `member_delivery_binding` shape carried inside [`SpaceMember`].
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
    /// `cx:event:*` ref to the service_acceptance event the recipient
    /// service signed for this binding (`explicit` / `invite` /
    /// `organization_policy` sources).
    #[serde(default)]
    pub service_acceptance_ref: Option<String>,
    /// Reference to the policy event that authorised this binding
    /// (`join_policy` / `organization_policy` / `space_policy` sources).
    #[serde(default)]
    pub policy_ref: Option<String>,
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

/// PATCH body for `/api/admin/v1/capabilities/{id}` — fine-grained
/// edits to an existing grant's constraints. All fields optional; the
/// admin only sends the keys that actually changed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCapabilityRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_write_allow: Option<Vec<String>>,
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
    pub created_at: Option<String>,
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
}

// ── Report / Moderation types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Report {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub space_id: Option<String>,
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
    pub space_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateInviteTokenRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses_allowed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
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
pub struct BlobInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub digest: Option<String>,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub owner_id: Option<String>,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub is_quarantined: bool,
    #[serde(default)]
    pub uploaded_at: Option<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerDescribeResBody {
    #[serde(default)]
    pub service_did: String,
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
    /// T1.4 — soland surfaces its dev-mode posture directly on
    /// `/api/v1/server/describe` (and `/health`). Sodmin uses this to
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
    /// T6.1 — profiles the server has been independently verified to
    /// implement against the spec test suite. Rendered as green
    /// "verified" chips.
    #[serde(default)]
    pub verified_profiles: Vec<String>,
    /// T6.1 — profiles the operator self-claims support for. Rendered
    /// as yellow "self-claimed" chips because they lack third-party
    /// verification.
    #[serde(default)]
    pub claimed_profiles: Vec<String>,
    /// T6.1 — experimental features the server exposes. Rendered as
    /// blue chips with an "unstable" warning.
    #[serde(default)]
    pub experimental_features: Vec<String>,
    /// T6.1 — compatibility surfaces (legacy / shim endpoints). Grey
    /// chips.
    #[serde(default)]
    pub compat_surfaces: Vec<String>,
    /// T1.4 / S5 — services whose plaintext bodies remain readable on
    /// this deployment (intentionally weak posture, dev-only).
    #[serde(default)]
    pub plaintext_visible_services: Vec<String>,
    /// T8.3 — production hardening checklist snapshot, mirrored from
    /// `/health`. Older servers that predate the field omit it.
    #[serde(default)]
    pub hardening: Option<HardeningStatus>,
}

/// T8.3 — production deployment hardening checklist snapshot.
///
/// Surfaced by every Contrix service (`soland`, `coauth`, `floria`,
/// `starid`, `teabay`) on `/health` and the corresponding describe
/// endpoint. The sodmin `/hardening` dashboard aggregates these into a
/// single board with green/red chips per check.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardeningStatus {
    #[serde(default)]
    pub development_mode: bool,
    #[serde(default)]
    pub tls_enabled: bool,
    #[serde(default)]
    pub csp_header_configured: bool,
    #[serde(default)]
    pub cors_strict: bool,
    #[serde(default)]
    pub secret_manager_in_use: bool,
    #[serde(default)]
    pub log_redaction_enabled: bool,
    #[serde(default)]
    pub admin_auth_mode: Option<String>,
    #[serde(default)]
    pub rate_limit_enabled: bool,
    #[serde(default)]
    pub provider_credential_rotation: Option<String>,
    #[serde(default)]
    pub checklist_score: u32,
    #[serde(default)]
    pub checklist_max: u32,
    #[serde(default)]
    pub warnings: Vec<String>,
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
    pub space_count: u64,
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

// ── Auth types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoginReqBody {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoginResponse {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub is_admin: Option<bool>,
}

// ── Profile types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileResBody {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

// ── Handle availability ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandleAvailabilityResult {
    #[serde(default)]
    pub available: bool,
    #[serde(default)]
    pub error: Option<String>,
}

// ── Handle management (T6.2 §2) ──

/// One row in `GET /api/admin/v1/handles`. Mirrors the `cx.handle.*` cell
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
    pub subject_did: Option<String>,
    #[serde(default)]
    pub assigned_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub last_reassignment_at: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Single audit event for `GET /api/admin/v1/handles/{id}/audit`. The
/// audit table is what T3.2 created — we surface the minimum the
/// operator needs to triage.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandleAuditEvent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub actor_did: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub previous_subject_did: Option<String>,
    #[serde(default)]
    pub new_subject_did: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandleReassignRequest {
    pub new_subject_did: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// ── Delivery binding policy (T6.2 §3) ──

/// Effective `cx.cell.realm.delivery_binding_policy` for a Realm
/// (security boundary; pre realm-rework these were called Spaces).
/// `allowed_recipient_services` and `binding_source_policy` are
/// operator-mutable; `policy_frontier` is written by the soland
/// reducer and is therefore read-only on the admin surface.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealmDeliveryBindingPolicy {
    /// Realm identifier (security boundary). The legacy wire name
    /// `space_id` is still accepted on the response shape.
    #[serde(default, alias = "space_id")]
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

/// Back-compat type alias for callers that have not migrated to the
/// realm-rework names yet.
// TODO(realm-rework): drop this alias once every callsite uses
// `RealmDeliveryBindingPolicy`.
pub type SpaceDeliveryBindingPolicy = RealmDeliveryBindingPolicy;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateDeliveryBindingPolicyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_recipient_services: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_source_policy: Option<String>,
}

/// One row in the per-Space "is each member routable?" check table. We
/// pull this straight from the Space members projection plus the
/// effective delivery_binding_policy.
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

// ── Realm link graph (R5.2, Round R1.2 — cx.realm.link projection) ──

/// One outbound / inbound typed link between two Realm boundaries.
/// Backed by `cx.realm.link` reducer_input events. Common `link_kind`
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

/// Surface for `cx.device.push_route` cells, grouped by principal so
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

#[cfg(test)]
mod tests {
    use super::{
        HandleRecord, ServerDescribeResBody, UpdateCapabilityRequest,
        UpdateDeliveryBindingPolicyRequest,
    };
    use serde_json::json;

    #[test]
    fn server_describe_accepts_principal_server_profile_status() {
        let describe: ServerDescribeResBody = serde_json::from_value(json!({
            "service_did": "did:web:soland.local",
            "service_type": "principal_server",
            "protocol_version": "1.0",
            "supported_profiles": ["cx.profile.principal_server.v1"],
            "supported_features": ["events.describe", "events.submit"],
            "supported_operations": ["cx.events.submit"],
            "supported_reducer_profiles": ["cx.reducer.v1"],
            "supported_schema_profiles": ["cx.schema.core.v1"],
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
            vec!["cx.reducer.v1".to_string()]
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
            "supported_profiles": ["cx.profile.auth_account.v1"],
            "auth_metadata": {
                "issuer_did": "did:web:auth.example.com#issuer",
                "oauth_issuer": "https://auth.example.com"
            },
            "identity_registry_resolver": {
                "mode": "delegated_resolver",
                "endpoint": "https://auth.example.com/api/v1/identity/resolve",
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
            "supported_profiles": ["cx.profile.identity_registry.v1"]
        }))
        .expect("supported_profiles should deserialize");

        assert_eq!(
            describe.supported_profiles,
            vec!["cx.profile.identity_registry.v1".to_string()]
        );
    }

    #[test]
    fn server_describe_reads_conformance_buckets() {
        let describe: ServerDescribeResBody = serde_json::from_value(json!({
            "service_did": "did:web:soland.local",
            "verified_profiles": ["cx.profile.principal_server.v1"],
            "claimed_profiles": ["cx.profile.identity_registry.v1"],
            "experimental_features": ["events.replay.v2"],
            "compat_surfaces": ["legacy.federation.v0"],
            "plaintext_visible_services": ["floria"],
        }))
        .expect("conformance buckets should deserialize");

        assert_eq!(
            describe.verified_profiles,
            vec!["cx.profile.principal_server.v1".to_string()]
        );
        assert_eq!(
            describe.claimed_profiles,
            vec!["cx.profile.identity_registry.v1".to_string()]
        );
        assert_eq!(
            describe.experimental_features,
            vec!["events.replay.v2".to_string()]
        );
        assert_eq!(
            describe.compat_surfaces,
            vec!["legacy.federation.v0".to_string()]
        );
        assert_eq!(
            describe.plaintext_visible_services,
            vec!["floria".to_string()]
        );
    }

    #[test]
    fn handle_record_round_trip() {
        let record: HandleRecord = serde_json::from_value(json!({
            "id": "h-1",
            "canonical_uri": "cx:handle:@alice",
            "aliases": ["@alice", "@alice.example"],
            "issuer_did": "did:web:auth.example.com",
            "subject_did": "did:key:zABC",
            "status": "active"
        }))
        .expect("handle record should deserialize");

        assert_eq!(record.canonical_uri, "cx:handle:@alice");
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
            fields_write_allow: Some(vec!["body.text".to_string()]),
            facets_allow: None,
            approval_required: Some(true),
        };
        let serialized = serde_json::to_string(&req).expect("serializes");
        assert!(serialized.contains("expires_at"));
        assert!(serialized.contains("fields_write_allow"));
        assert!(serialized.contains("approval_required"));
        assert!(!serialized.contains("facets_allow"));
    }
}
