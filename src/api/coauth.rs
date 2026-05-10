//! coauth admin API client surface.
//!
//! Wire shapes that already live in the shared `coauth-admin-types`
//! crate are imported / re-exported from there (see the `pub use
//! coauth_admin_types::…` block below the local types). Anything still
//! defined inline here is on the migration list — when the corresponding
//! coauth admin handler graduates from `serde_json::json!` literals to
//! a typed response, lift the struct into `coauth-admin-types` and turn
//! the local copy into a re-export, in lock-step with this client.
//!
//! TODO(a0-shared-crate): finish migrating the remaining inline DTOs
//! (account summary / DID binding / claim / session grant / bridge
//! describe / integration manifest) into `coauth-admin-types` so this
//! file is reduced to API verb wrappers plus a re-export block.
//!
//! Round 32 (C32.7): the connector-health and notification-channel /
//! notification-template DTOs moved into
//! `coauth_admin_types::{connector_health, notification_admin}`. The
//! sodmin-side mirror structs (`CoauthConnectorHealth`,
//! `CoauthNotificationChannel`, `CoauthNotificationTemplate`) had
//! drifted out of wire-shape parity with the coauth backend (invented
//! `id`/`is_healthy`/`last_error`/`channel_type`/`updated_at` fields
//! that the server never emitted) — they are now removed and consumers
//! import from the shared crate so rustc enforces the contract.
//!
//! Round 28 (A0 switch-over): the recovery / federation / space-policy /
//! applets-admin DTOs that previously lived inline under
//! `crate::types::{recovery,federation_status,space_policy,applets_admin}`
//! have moved into `coauth_admin_types::{recovery_admin, federation_admin,
//! space_policy_admin, applets_admin}` and the sodmin-side mirror modules
//! were deleted. Consumers import from the shared crate directly —
//! rustc enforces wire-shape parity across the coauth backend and the
//! admin SPA from this point forward.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::client::{api_client, build_url};
use crate::utils::error::HttpError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthViewer {
    #[serde(default)]
    pub sub: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthUser {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub is_locked: bool,
    #[serde(default)]
    pub is_deactivated: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthAuditEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub operation: String,
    #[serde(default)]
    pub actor_user_id: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthOAuth2Session {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub human_name: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthPersonalSession {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_active_at: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthUpstreamProvider {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub issuer: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthUpstreamLink {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthRegistrationToken {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub uses_allowed: Option<u64>,
    #[serde(default)]
    pub uses_completed: u64,
    #[serde(default)]
    pub uses_pending: u64,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub is_revoked: bool,
}

// Connector-health and notification (channel + template) wire shapes
// are sourced from `coauth-admin-types` so rustc enforces parity with
// the backend handlers. The aliases keep sodmin's existing call-site
// names (`Coauth*`) intact while the actual struct definitions live in
// the shared crate.
pub use coauth_admin_types::ConnectorHealthRow as CoauthConnectorHealth;
pub use coauth_admin_types::NotificationChannelStatus as CoauthNotificationChannel;
pub use coauth_admin_types::NotificationTemplateEntry as CoauthNotificationTemplate;
pub use coauth_admin_types::PublishTemplateRequest as CoauthPublishTemplateRequest;
pub use coauth_admin_types::PublishedTemplateResponse as CoauthPublishedTemplate;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthAccountSummary {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub primary_did: Option<String>,
    #[serde(default)]
    pub is_locked: bool,
    #[serde(default)]
    pub is_deactivated: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub bridge_status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthManagedDidBinding {
    #[serde(default)]
    pub did: String,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub last_verified_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthAccountClaim {
    #[serde(default)]
    pub claim_type: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthSessionGrantSummary {
    #[serde(default)]
    pub grant_id: String,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub issued_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthRiskActionHook {
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub approval_mode: String,
    #[serde(default)]
    pub todo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthAdminBridgeDescribe {
    #[serde(default)]
    pub contract: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub api_base_path: String,
    #[serde(default)]
    pub accounts_path: String,
    #[serde(default)]
    pub account_detail_path_template: String,
    #[serde(default)]
    pub account_dids_path_template: String,
    #[serde(default)]
    pub account_claims_path_template: String,
    #[serde(default)]
    pub account_session_grants_path_template: String,
    #[serde(default)]
    pub risk_action_path_template: String,
    #[serde(default)]
    pub risk_action_current_path_template: String,
    #[serde(default)]
    pub risk_action_history_path_template: String,
    #[serde(default)]
    pub risk_action_approve_path_template: String,
    #[serde(default)]
    pub risk_action_execute_path_template: String,
    #[serde(default)]
    pub risk_action_state_store_kind: String,
    #[serde(default)]
    pub risk_action_approval_mode: String,
    #[serde(default)]
    pub risk_action_examples: CoauthAdminBridgeRiskActionExamples,
    #[serde(default)]
    pub todos: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthAdminBridgeRiskActionExamples {
    #[serde(default)]
    pub proposal_request: Value,
    #[serde(default)]
    pub approve_request: Value,
    #[serde(default)]
    pub execute_request: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthIntegrationManifest {
    #[serde(default)]
    pub contract: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub service: String,
    #[serde(default)]
    pub service_kind: String,
    #[serde(default)]
    pub api_base_path: String,
    #[serde(default)]
    pub describe_path: String,
    #[serde(default)]
    pub dependencies: Vec<CoauthIntegrationDependency>,
    #[serde(default)]
    pub surfaces: Vec<CoauthIntegrationSurface>,
    #[serde(default)]
    pub todos: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthIntegrationDependency {
    #[serde(default)]
    pub service: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub required_contract: String,
    #[serde(default)]
    pub discovery_path: String,
    #[serde(default)]
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthIntegrationSurface {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub contract: String,
    #[serde(default)]
    pub stability: String,
    #[serde(default)]
    pub todo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthRecoveryBridgeDescribe {
    #[serde(default)]
    pub contract: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub recovery_start_path: String,
    #[serde(default)]
    pub recovery_status_path: String,
    #[serde(default)]
    pub recovery_resend_path: String,
    #[serde(default)]
    pub recovery_principal_snapshot_path: String,
    #[serde(default)]
    pub recovery_principal_cache_status_path: String,
    #[serde(default)]
    pub recovery_principal_cache_refresh_path: String,
    #[serde(default)]
    pub recovery_principal_cache_queue_path: String,
    #[serde(default)]
    pub recovery_principal_cache_complete_path: String,
    #[serde(default)]
    pub recovery_principal_cache_fail_path: String,
    #[serde(default)]
    pub recovery_principal_cache_policy_path: String,
    #[serde(default)]
    pub recovery_principal_cache_retry_path: String,
    #[serde(default)]
    pub recovery_principal_cache_invalidate_path: String,
    #[serde(default)]
    pub recovery_principal_cache_failures_path: String,
    #[serde(default)]
    pub recovery_principal_cache_upstream_path: String,
    #[serde(default)]
    pub recovery_principal_cache_upstream_probe_path: String,
    #[serde(default)]
    pub recovery_principal_cache_upstream_bind_path: String,
    #[serde(default)]
    pub key_backup_rest_base: String,
    #[serde(default)]
    pub key_backup_schema: String,
    #[serde(default)]
    pub device_message_schema: String,
    #[serde(default)]
    pub principal_recovery_contract_stack_path: String,
    #[serde(default)]
    pub principal_recovery_stack_bundle_path: String,
    #[serde(default)]
    pub principal_recovery_discovery_path: String,
    #[serde(default)]
    pub principal_recovery_readiness_path: String,
    #[serde(default)]
    pub principal_device_messages_describe_path: String,
    #[serde(default)]
    pub principal_key_backups_describe_path: String,
    #[serde(default)]
    pub principal_restore_state_describe_path: String,
    #[serde(default)]
    pub principal_restore_state_export_path: String,
    #[serde(default)]
    pub principal_restore_state_import_path: String,
    #[serde(default)]
    pub principal_restore_state_durability_path: String,
    #[serde(default)]
    pub principal_restore_state_checkpoint_collection_path: String,
    #[serde(default)]
    pub principal_restore_start_path: String,
    #[serde(default)]
    pub principal_restore_describe_path: String,
    #[serde(default)]
    pub principal_restore_ticket_collection_path: String,
    #[serde(default)]
    pub principal_restore_ticket_path: String,
    #[serde(default)]
    pub principal_restore_ticket_advance_path: String,
    #[serde(default)]
    pub principal_restore_ticket_resume_path: String,
    #[serde(default)]
    pub principal_restore_ticket_cancel_path: String,
    #[serde(default)]
    pub principal_restore_ticket_retry_path: String,
    #[serde(default)]
    pub principal_restore_approval_status_path: String,
    #[serde(default)]
    pub principal_restore_approval_submit_path: String,
    #[serde(default)]
    pub principal_restore_executor_status_path: String,
    #[serde(default)]
    pub principal_restore_executor_enqueue_path: String,
    #[serde(default)]
    pub principal_restore_executor_start_path: String,
    #[serde(default)]
    pub principal_restore_executor_complete_path: String,
    #[serde(default)]
    pub principal_restore_result_path: String,
    #[serde(default)]
    pub principal_restore_receipt_path: String,
    #[serde(default)]
    pub principal_restore_materialized_device_handoff_path: String,
    #[serde(default)]
    pub principal_restore_bundle_path: String,
    #[serde(default)]
    pub principal_restore_activity_path: String,
    #[serde(default)]
    pub principal_restore_timeline_path: String,
    #[serde(default)]
    pub principal_restore_audit_feed_path: String,
    #[serde(default)]
    pub principal_recovery_live_snapshot_path: String,
    #[serde(default)]
    pub principal_authz_describe_path: String,
    #[serde(default)]
    pub principal_authz_check_path: String,
    #[serde(default)]
    pub principal_policy_describe_path: String,
    #[serde(default)]
    pub principal_policy_collection_path: String,
    #[serde(default)]
    pub principal_policy_item_path: String,
    #[serde(default)]
    pub verification_event_kinds: Vec<String>,
    #[serde(default)]
    pub recovery_modes: Vec<String>,
    #[serde(default)]
    pub example_backup_payload: Value,
    #[serde(default)]
    pub recovery_restore_examples: Value,
    #[serde(default)]
    pub recovery_authz_examples: Value,
    #[serde(default)]
    pub todos: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthAccountDetail {
    #[serde(default)]
    pub account: CoauthAccountSummary,
    #[serde(default)]
    pub managed_dids: Vec<CoauthManagedDidBinding>,
    #[serde(default)]
    pub claims: Vec<CoauthAccountClaim>,
    #[serde(default)]
    pub session_grants: Vec<CoauthSessionGrantSummary>,
    #[serde(default)]
    pub risk_action_current: CoauthAccountRiskActionCurrentState,
    #[serde(default)]
    pub risk_action_history: Vec<CoauthAccountRiskActionHistoryEntry>,
    #[serde(default)]
    pub risk_action_hook: CoauthRiskActionHook,
    #[serde(default)]
    pub admin_bridge: CoauthAdminBridgeDescribe,
    #[serde(default)]
    pub recovery_bridge: CoauthRecoveryBridgeDescribe,
    #[serde(default)]
    pub integration_manifest: CoauthIntegrationManifest,
}

// Risk-action wire shapes are sourced from `coauth-admin-types` so drift
// between sodmin's request bodies / response decoders and coauth's typed
// handlers is caught at compile time. The aliases below preserve the
// `Coauth*` names that sodmin's UI components have been using.
pub use coauth_admin_types::AccountRiskActionApprovalRequest as CoauthAccountRiskActionApprovalDraft;
pub use coauth_admin_types::AccountRiskActionApprovalResponse as CoauthAccountRiskActionApproval;
pub use coauth_admin_types::AccountRiskActionCurrentResponse as CoauthAccountRiskActionCurrentState;
pub use coauth_admin_types::AccountRiskActionExecuteRequest as CoauthAccountRiskActionExecuteDraft;
pub use coauth_admin_types::AccountRiskActionHistoryResponse as CoauthAccountRiskActionHistoryEnvelopeShared;
pub use coauth_admin_types::AccountRiskActionProposalRequest as CoauthAccountRiskActionDraft;
pub use coauth_admin_types::AccountRiskActionProposalResponse as CoauthAccountRiskActionProposal;
pub use coauth_admin_types::AccountRiskActionTransitionRecord as CoauthAccountRiskActionHistoryEntry;

// `CoauthAccountRiskActionProposal` and `CoauthAccountRiskActionApproval`
// are now sourced from `coauth_admin_types` via the re-exports above.

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthAccountRiskActionExecute {
    #[serde(default)]
    pub state_record_id: String,
    #[serde(default)]
    pub proposal_id: String,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub ticket: Option<String>,
    #[serde(default)]
    pub previous_state: String,
    #[serde(default)]
    pub execution_state: String,
    #[serde(default)]
    pub state_revision: u64,
    #[serde(default)]
    pub transition_kind: String,
    #[serde(default)]
    pub executed_at: Option<String>,
    #[serde(default)]
    pub execution_mode: String,
    #[serde(default)]
    pub execution_note: Option<String>,
    #[serde(default)]
    pub mutation_endpoint: String,
    #[serde(default)]
    pub allowed_next_transitions: Vec<String>,
    #[serde(default)]
    pub state_store_kind: String,
    #[serde(default)]
    pub todo: String,
}

// `CoauthAccountRiskActionHistoryEntry` and the `{data: [...]}` envelope
// shape are now sourced from `coauth_admin_types` via the re-exports
// above; the local definitions used to live here.

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAccountClaimsEnvelope {
    #[serde(default)]
    data: Vec<CoauthAccountClaim>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAccountSessionGrantsEnvelope {
    #[serde(default)]
    data: Vec<CoauthSessionGrantSummary>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAdminPaginatedEnvelope<T> {
    #[serde(default)]
    data: Option<Vec<CoauthAdminResource<T>>>,
    #[serde(default)]
    meta: CoauthAdminPaginationMeta,
    #[serde(default)]
    links: coauth_admin_types::PaginationLinks,
}

// Single-resource envelope is now sourced from `coauth-admin-types` so
// the wire shape matches what the backend actually emits (the previous
// hand-written shape was a strict subset). The aliases keep sodmin's
// callers using the local names while sharing the typed wrapper.
type CoauthAdminSingleEnvelope<T> = coauth_admin_types::SingleResponse<T>;
type CoauthAdminResource<T> = coauth_admin_types::SingleResource<T>;

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAdminPaginationMeta {
    #[serde(default)]
    count: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAdminAccountRecord {
    #[serde(default)]
    username: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    avatar_url: Option<String>,
    #[serde(default)]
    preferred_locale: Option<String>,
    #[serde(default)]
    primary_principal_did: Option<String>,
    #[serde(default)]
    principal_dids: Vec<String>,
    #[serde(default)]
    admin: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAdminDidBindingsEnvelope {
    #[serde(default)]
    data: Vec<CoauthAdminDidBindingRecord>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAdminDidBindingRecord {
    #[serde(default)]
    did: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    verification_status: String,
    #[serde(default)]
    primary: bool,
    #[serde(default)]
    active: bool,
    #[serde(default)]
    last_verified_at: Option<String>,
}

// ── API methods ──

pub async fn get_viewer() -> Result<CoauthViewer, HttpError> {
    api_client("/api/v1/viewer", "GET", None).await
}

pub async fn list_users(
    page: u64,
    per_page: u64,
    search: &str,
) -> Result<PaginatedResponse<CoauthUser>, HttpError> {
    let url = build_url(
        "/contrix/admin/v1/users",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
            ("search", search),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn get_user(id: &str) -> Result<CoauthUser, HttpError> {
    let url = format!("/contrix/admin/v1/users/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn update_user(id: &str, patch: &serde_json::Value) -> Result<CoauthUser, HttpError> {
    let url = format!("/contrix/admin/v1/users/{}", urlencoding::encode(id));
    api_client(&url, "PATCH", Some(patch.to_string())).await
}

pub async fn set_user_password(id: &str, password: &str) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/users/{}/set-password",
        urlencoding::encode(id)
    );
    let body = serde_json::json!({ "password": password });
    api_client(&url, "POST", Some(body.to_string())).await
}

/// Multi-dimensional filter for `/admin/v1/audit-feed` queries. Empty
/// fields are dropped before encoding so the wire form only carries
/// what the operator actually filtered on.
#[derive(Debug, Clone, Default)]
pub struct AuditFeedFilter {
    pub operation: Option<String>,
    pub actor_user_id: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
}

impl AuditFeedFilter {
    fn into_query(self) -> Vec<(&'static str, String)> {
        [
            ("operation", self.operation),
            ("actor_user_id", self.actor_user_id),
            ("target_type", self.target_type),
            ("target_id", self.target_id),
            ("since", self.since),
            ("until", self.until),
        ]
        .into_iter()
        .filter_map(|(k, v)| {
            v.map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .map(|s| (k, s))
        })
        .collect()
    }
}

pub async fn list_audit_feed(
    page: u64,
    per_page: u64,
    filter: AuditFeedFilter,
) -> Result<PaginatedResponse<CoauthAuditEntry>, HttpError> {
    let page_str = page.to_string();
    let per_page_str = per_page.to_string();
    let mut params: Vec<(&str, &str)> = vec![
        ("page", page_str.as_str()),
        ("per_page", per_page_str.as_str()),
    ];
    let owned = filter.into_query();
    for (k, v) in owned.iter() {
        params.push((k, v.as_str()));
    }
    let url = build_url("/contrix/admin/v1/audit-feed", &params)?;
    api_client(&url, "GET", None).await
}

pub async fn list_oauth2_sessions(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthOAuth2Session>, HttpError> {
    let url = build_url(
        "/contrix/admin/v1/oauth2-sessions",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn finish_oauth2_session(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/oauth2-sessions/{}/finish",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn list_personal_sessions(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthPersonalSession>, HttpError> {
    let url = build_url(
        "/contrix/admin/v1/personal-sessions",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn create_personal_session(name: &str) -> Result<CoauthPersonalSession, HttpError> {
    let body = serde_json::json!({ "name": name });
    api_client(
        "/contrix/admin/v1/personal-sessions",
        "POST",
        Some(body.to_string()),
    )
    .await
}

pub async fn revoke_personal_session(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/personal-sessions/{}/revoke",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn regenerate_personal_session(id: &str) -> Result<CoauthPersonalSession, HttpError> {
    let url = format!(
        "/contrix/admin/v1/personal-sessions/{}/regenerate",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn list_upstream_providers(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthUpstreamProvider>, HttpError> {
    let url = build_url(
        "/contrix/admin/v1/upstream-oauth-providers",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn create_upstream_provider(
    provider: &serde_json::Value,
) -> Result<CoauthUpstreamProvider, HttpError> {
    api_client(
        "/contrix/admin/v1/upstream-oauth-providers",
        "POST",
        Some(provider.to_string()),
    )
    .await
}

pub async fn delete_upstream_provider(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/upstream-oauth-providers/{}",
        urlencoding::encode(id)
    );
    api_client(&url, "DELETE", None).await
}

pub async fn toggle_upstream_provider(id: &str, enable: bool) -> Result<(), HttpError> {
    let action = if enable { "enable" } else { "disable" };
    let url = format!(
        "/contrix/admin/v1/upstream-oauth-providers/{}/{}",
        urlencoding::encode(id),
        action
    );
    api_client(&url, "POST", None).await
}

pub async fn list_upstream_links(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthUpstreamLink>, HttpError> {
    let url = build_url(
        "/contrix/admin/v1/upstream-oauth-links",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn delete_upstream_link(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/upstream-oauth-links/{}",
        urlencoding::encode(id)
    );
    api_client(&url, "DELETE", None).await
}

pub async fn list_registration_tokens(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthRegistrationToken>, HttpError> {
    let url = build_url(
        "/contrix/admin/v1/user-registration-tokens",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn create_registration_token(
    uses_allowed: Option<u64>,
) -> Result<CoauthRegistrationToken, HttpError> {
    let body = serde_json::json!({ "uses_allowed": uses_allowed });
    api_client(
        "/contrix/admin/v1/user-registration-tokens",
        "POST",
        Some(body.to_string()),
    )
    .await
}

pub async fn revoke_registration_token(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/user-registration-tokens/{}/revoke",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn get_connector_health() -> Result<Vec<CoauthConnectorHealth>, HttpError> {
    let resp: coauth_admin_types::ConnectorHealthResponse =
        api_client("/contrix/admin/v1/connector-health", "GET", None).await?;
    Ok(resp.providers)
}

pub async fn list_notification_channels() -> Result<Vec<CoauthNotificationChannel>, HttpError> {
    let resp: coauth_admin_types::NotificationChannelsResponse =
        api_client("/contrix/admin/v1/notification-channels", "GET", None).await?;
    Ok(resp.channels)
}

/// Returns the static catalog of known notification template keys. The
/// backend endpoint is non-paginated — it returns the full known-keys
/// list every call — so the sodmin UI now flat-lists the entries
/// instead of pretending there's a paging cursor.
pub async fn list_notification_templates() -> Result<Vec<CoauthNotificationTemplate>, HttpError> {
    let resp: coauth_admin_types::NotificationTemplatesResponse =
        api_client("/contrix/admin/v1/notification-templates", "GET", None).await?;
    Ok(resp.templates)
}

/// Publish a notification template version. The request body carries
/// the template key + channel + body (and optional subject + locale);
/// the response is the persisted row.
pub async fn publish_notification_template(
    request: &CoauthPublishTemplateRequest,
) -> Result<CoauthPublishedTemplate, HttpError> {
    let body = serde_json::to_string(request).map_err(|e| HttpError {
        status: 0,
        message: format!("serialize publish-template request: {e}"),
        body: None,
        request_id: None,
        retry_after_ms: None,
    })?;
    api_client(
        "/contrix/admin/v1/notification-templates/publish",
        "POST",
        Some(body),
    )
    .await
}

pub async fn list_accounts(
    page: u64,
    per_page: u64,
    search: &str,
) -> Result<PaginatedResponse<CoauthAccountSummary>, HttpError> {
    let requested = page.max(1).saturating_mul(per_page.max(1));
    let url = build_url(
        "/contrix/admin/v1/accounts",
        &[
            ("filter[search]", search),
            ("page[first]", &requested.to_string()),
            ("count", "true"),
        ],
    )?;
    let resp: CoauthAdminPaginatedEnvelope<CoauthAdminAccountRecord> =
        api_client(&url, "GET", None).await?;
    let start = page.saturating_sub(1).saturating_mul(per_page) as usize;
    let end = start.saturating_add(per_page as usize);
    let summaries: Vec<CoauthAccountSummary> = resp
        .data
        .unwrap_or_default()
        .into_iter()
        .map(map_admin_account_summary_resource)
        .collect();
    let total = resp.meta.count.unwrap_or(summaries.len() as u64);
    Ok(PaginatedResponse {
        data: summaries
            .into_iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect(),
        total,
    })
}

/// Cursor-paginated account list. Differs from `list_accounts` in that
/// the caller passes back the opaque `cursor` it received from the prior
/// page's `links.next` instead of converting to / from a 1-based page
/// number. Use this for any new UI; `list_accounts` is preserved for the
/// legacy index-page surface.
///
/// Wire shape: `?filter[search]=…&filter[handle]=…&filter[display_name]=…
/// &cursor={base64url}&limit=N&count=true`. The base64url cursor is the
/// raw value coauth published; decoding/encoding on the wire is the
/// server's responsibility.
pub async fn list_accounts_cursor(
    cursor: Option<&str>,
    limit: u64,
    search: &str,
    filter: &AccountListFilter,
) -> Result<CursorPage<CoauthAccountSummary>, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![
        ("filter[search]", search),
        ("filter[handle]", filter.handle.trim()),
        ("filter[display_name]", filter.display_name.trim()),
        ("limit", limit_str.as_str()),
        ("count", "true"),
    ];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url("/contrix/admin/v1/accounts", &params)?;
    let resp: CoauthAdminPaginatedEnvelope<CoauthAdminAccountRecord> =
        api_client(&url, "GET", None).await?;
    let summaries: Vec<CoauthAccountSummary> = resp
        .data
        .unwrap_or_default()
        .into_iter()
        .map(map_admin_account_summary_resource)
        .collect();
    let next_cursor = resp
        .links
        .next
        .as_deref()
        .and_then(extract_cursor_param);
    let prev_cursor = resp
        .links
        .prev
        .as_deref()
        .and_then(extract_cursor_param);
    Ok(CursorPage {
        data: summaries,
        next_cursor,
        prev_cursor,
        total: resp.meta.count,
    })
}

pub async fn get_account_detail(id: &str) -> Result<CoauthAccountDetail, HttpError> {
    let bridge_url = "/contrix/admin/v1/bridge/describe";
    let recovery_url = "/contrix/api/v1/auth/recovery/describe";
    let integration_manifest_url = "/contrix/api/v1/integration/describe";
    let summary_url = format!("/contrix/admin/v1/accounts/{}", urlencoding::encode(id));
    let dids_url = format!("/contrix/admin/v1/accounts/{}/dids", urlencoding::encode(id));
    let claims_url = format!("/contrix/admin/v1/accounts/{}/claims", urlencoding::encode(id));
    let grants_url = format!(
        "/contrix/admin/v1/accounts/{}/session-grants",
        urlencoding::encode(id)
    );
    let current_url = format!(
        "/contrix/admin/v1/accounts/{}/risk-action/current",
        urlencoding::encode(id)
    );
    let history_url = format!(
        "/contrix/admin/v1/accounts/{}/risk-action/history",
        urlencoding::encode(id)
    );
    let summary: CoauthAdminSingleEnvelope<CoauthAdminAccountRecord> =
        api_client(&summary_url, "GET", None).await?;
    let dids: CoauthAdminDidBindingsEnvelope = api_client(&dids_url, "GET", None).await?;
    let claims: CoauthAccountClaimsEnvelope = api_client(&claims_url, "GET", None).await?;
    let session_grants: CoauthAccountSessionGrantsEnvelope =
        api_client(&grants_url, "GET", None).await?;
    let bridge: CoauthAdminBridgeDescribe = api_client(bridge_url, "GET", None).await?;
    let recovery_bridge: CoauthRecoveryBridgeDescribe = api_client(recovery_url, "GET", None).await?;
    let integration_manifest: CoauthIntegrationManifest =
        api_client(integration_manifest_url, "GET", None).await?;
    let current: CoauthAdminSingleEnvelope<CoauthAccountRiskActionCurrentState> =
        api_client(&current_url, "GET", None).await?;
    let history: CoauthAccountRiskActionHistoryEnvelopeShared =
        api_client(&history_url, "GET", None).await?;
    let account = map_admin_account_summary_resource(summary.data);
    Ok(CoauthAccountDetail {
        claims: claims.data,
        session_grants: session_grants.data,
        risk_action_current: current.data.attributes,
        risk_action_history: history.data,
        managed_dids: dids
            .data
            .into_iter()
            .map(map_admin_did_binding)
            .collect(),
        account,
        risk_action_hook: CoauthRiskActionHook {
            endpoint: bridge
                .risk_action_path_template
                .replace("{account_id}", id),
            approval_mode: bridge.risk_action_approval_mode.clone(),
            todo: if bridge.todos.is_empty() {
                "Coauth account admin bridge does not yet publish scaffold TODO items.".to_string()
            } else {
                bridge.todos.join(" ")
            },
        },
        admin_bridge: bridge,
        recovery_bridge,
        integration_manifest,
    })
}

/// Add a managed DID binding to an account. The `control_proof` is an
/// opaque blob (typically a signed challenge) the backend forwards to
/// the DID resolver — sodmin does not interpret it client-side.
pub async fn add_account_did_binding(
    account_id: &str,
    did: &str,
    control_proof: &str,
) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/accounts/{}/dids",
        urlencoding::encode(account_id)
    );
    let body = serde_json::json!({
        "did": did,
        "control_proof": control_proof,
    });
    let _: serde_json::Value = api_client(&url, "POST", Some(body.to_string())).await?;
    Ok(())
}

/// Remove a managed DID binding from an account. The DID is part of the
/// path so the request body is empty.
pub async fn remove_account_did_binding(
    account_id: &str,
    did: &str,
) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/accounts/{}/dids/{}",
        urlencoding::encode(account_id),
        urlencoding::encode(did),
    );
    let _: serde_json::Value = api_client(&url, "DELETE", None).await?;
    Ok(())
}

/// Revoke a single claim attached to the account. The claim is keyed by
/// its `claim_type` (e.g. `email`, `principal_did`); coauth's claims
/// admin routes accept the type as a path segment.
pub async fn revoke_account_claim(
    account_id: &str,
    claim_type: &str,
) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/accounts/{}/claims/{}/revoke",
        urlencoding::encode(account_id),
        urlencoding::encode(claim_type),
    );
    let _: serde_json::Value = api_client(&url, "POST", None).await?;
    Ok(())
}

pub async fn submit_account_risk_action(
    id: &str,
    draft: &CoauthAccountRiskActionDraft,
) -> Result<CoauthAccountRiskActionProposal, HttpError> {
    let url = format!(
        "/contrix/admin/v1/accounts/{}/risk-action",
        urlencoding::encode(id)
    );
    let body = serde_json::json!({
        "action": draft.action,
        "reason": draft.reason,
        "ticket": draft.ticket,
        "approved_by": draft.approved_by,
    });
    api_client(&url, "POST", Some(body.to_string())).await
}

pub async fn approve_account_risk_action(
    id: &str,
    proposal_id: &str,
    draft: &CoauthAccountRiskActionApprovalDraft,
) -> Result<CoauthAccountRiskActionApproval, HttpError> {
    let url = format!(
        "/contrix/admin/v1/accounts/{}/risk-action/{}/approve",
        urlencoding::encode(id),
        urlencoding::encode(proposal_id)
    );
    let body = serde_json::json!({
        "action": draft.action,
        "ticket": draft.ticket,
        "approved_by": draft.approved_by,
        "approval_note": draft.approval_note,
    });
    api_client(&url, "POST", Some(body.to_string())).await
}

pub async fn execute_account_risk_action(
    id: &str,
    proposal_id: &str,
    draft: &CoauthAccountRiskActionExecuteDraft,
) -> Result<CoauthAccountRiskActionExecute, HttpError> {
    let url = format!(
        "/contrix/admin/v1/accounts/{}/risk-action/{}/execute",
        urlencoding::encode(id),
        urlencoding::encode(proposal_id)
    );
    let body = serde_json::json!({
        "action": draft.action,
        "ticket": draft.ticket,
        "execution_note": draft.execution_note,
    });
    api_client(&url, "POST", Some(body.to_string())).await
}

pub async fn lock_account(id: &str) -> Result<(), HttpError> {
    let url = format!("/contrix/admin/v1/accounts/{}/lock", urlencoding::encode(id));
    let _: serde_json::Value = api_client(&url, "POST", None).await?;
    Ok(())
}

pub async fn disable_account(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/accounts/{}/disable",
        urlencoding::encode(id)
    );
    let _: serde_json::Value = api_client(&url, "POST", None).await?;
    Ok(())
}

pub async fn erase_account(id: &str) -> Result<(), HttpError> {
    let url = format!("/contrix/admin/v1/accounts/{}/erase", urlencoding::encode(id));
    let _: serde_json::Value = api_client(&url, "POST", None).await?;
    Ok(())
}

pub async fn reset_account_recovery(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/accounts/{}/reset-recovery",
        urlencoding::encode(id)
    );
    let _: serde_json::Value = api_client(&url, "POST", None).await?;
    Ok(())
}

fn map_user_to_account_summary(user: CoauthUser) -> CoauthAccountSummary {
    let primary_did = placeholder_primary_did(&user.id);
    CoauthAccountSummary {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        email: user.email,
        primary_did: Some(primary_did),
        is_locked: user.is_locked,
        is_deactivated: user.is_deactivated,
        created_at: user.created_at,
        updated_at: user.updated_at,
        bridge_status: "legacy_user_bridge+placeholder_account_contract".to_string(),
    }
}

fn map_admin_account_summary_resource(
    resource: CoauthAdminResource<CoauthAdminAccountRecord>,
) -> CoauthAccountSummary {
    let attributes = resource.attributes;
    let is_locked = attributes.status == "locked";
    let is_deactivated = attributes.status == "disabled";
    CoauthAccountSummary {
        id: resource.id,
        username: Some(attributes.username),
        display_name: attributes.display_name,
        avatar_url: attributes.avatar_url,
        email: None,
        primary_did: attributes.primary_principal_did.or_else(|| attributes.principal_dids.first().cloned()),
        is_locked,
        is_deactivated,
        created_at: attributes.created_at,
        updated_at: attributes.updated_at,
        bridge_status: if attributes.admin {
            "coauth_admin_accounts_v1+admin".to_string()
        } else {
            "coauth_admin_accounts_v1".to_string()
        },
    }
}

fn map_admin_did_binding(binding: CoauthAdminDidBindingRecord) -> CoauthManagedDidBinding {
    CoauthManagedDidBinding {
        did: binding.did,
        method: Some(binding.kind),
        state: Some(format!(
            "{}:{}:{}",
            binding.state,
            binding.verification_status,
            if binding.primary { "primary" } else { "secondary" }
        )),
        last_verified_at: binding.last_verified_at,
    }
}

fn admin_account_claims(account: &CoauthAccountSummary) -> Vec<CoauthAccountClaim> {
    let mut claims = Vec::new();
    if let Some(handle) = account.username.as_ref().filter(|value| !value.is_empty()) {
        claims.push(CoauthAccountClaim {
            claim_type: "handle".to_string(),
            value: Some(handle.clone()),
            state: Some("resolved_from_account_admin".to_string()),
            source: Some("coauth_admin_accounts_v1".to_string()),
        });
    }
    if let Some(primary_did) = account.primary_did.as_ref().filter(|value| !value.is_empty()) {
        claims.push(CoauthAccountClaim {
            claim_type: "principal_did".to_string(),
            value: Some(primary_did.clone()),
            state: Some("resolved_from_account_admin".to_string()),
            source: Some("coauth_admin_accounts_v1".to_string()),
        });
    }
    claims
}

fn bridge_managed_dids(user: &CoauthUser) -> Vec<CoauthManagedDidBinding> {
    vec![CoauthManagedDidBinding {
        did: placeholder_primary_did(&user.id),
        method: Some("did:web".to_string()),
        state: Some(if user.is_deactivated {
            "deactivated_placeholder".to_string()
        } else if user.is_locked {
            "locked_placeholder".to_string()
        } else {
            "bridge_placeholder".to_string()
        }),
        last_verified_at: user.updated_at.clone().or_else(|| user.created_at.clone()),
    }]
}

fn bridge_account_claims(user: &CoauthUser) -> Vec<CoauthAccountClaim> {
    let mut claims = Vec::new();
    if let Some(username) = user.username.as_ref().filter(|value| !value.is_empty()) {
        claims.push(CoauthAccountClaim {
            claim_type: "handle".to_string(),
            value: Some(username.clone()),
            state: Some("bridge_placeholder".to_string()),
            source: Some("legacy_user_bridge".to_string()),
        });
    }
    if let Some(email) = user.email.as_ref().filter(|value| !value.is_empty()) {
        claims.push(CoauthAccountClaim {
            claim_type: "email".to_string(),
            value: Some(email.clone()),
            state: Some("bridge_placeholder".to_string()),
            source: Some("legacy_user_bridge".to_string()),
        });
    }
    if claims.is_empty() {
        claims.push(CoauthAccountClaim {
            claim_type: "account_id".to_string(),
            value: Some(user.id.clone()),
            state: Some("bridge_placeholder".to_string()),
            source: Some("legacy_user_bridge".to_string()),
        });
    }
    claims
}

fn bridge_session_grants(user: &CoauthUser) -> Vec<CoauthSessionGrantSummary> {
    vec![CoauthSessionGrantSummary {
        grant_id: format!("grant-bridge-preview-{}", bridge_slug(&user.id)),
        subject: Some(user.id.clone()),
        scope: Some("urn:contrix:principal-server:session.bind".to_string()),
        state: Some(if user.is_deactivated {
            "disabled_placeholder".to_string()
        } else {
            "scaffold_preview".to_string()
        }),
        issued_at: user.updated_at.clone().or_else(|| user.created_at.clone()),
    }]
}

fn placeholder_primary_did(account_id: &str) -> String {
    format!("did:web:coauth.invalid:accounts:{}", bridge_slug(account_id))
}

fn bridge_slug(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
}

/// Cursor-shaped pagination result. `next_cursor` is `Some` when the
/// server links another page; `prev_cursor` mirrors it for backward
/// navigation. Both are opaque base64url strings produced by coauth and
/// fed back unchanged on the next request.
#[derive(Debug, Clone, Default)]
pub struct CursorPage<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
    /// Total when the server populated `meta.count` (best-effort).
    pub total: Option<u64>,
}

/// Filter inputs accepted by `list_accounts_cursor`. Empty strings are
/// dropped before encoding so the wire form only carries what the
/// operator actually filtered on.
#[derive(Debug, Clone, Default)]
pub struct AccountListFilter {
    pub handle: String,
    pub display_name: String,
}

impl AccountListFilter {
    pub fn is_empty(&self) -> bool {
        self.handle.trim().is_empty() && self.display_name.trim().is_empty()
    }
}

/// Parse the `cursor=…` query parameter out of a JSON:API `links.next` /
/// `links.prev` URL. Returns `None` when the link is absent or has no
/// cursor.
fn extract_cursor_param(link: &str) -> Option<String> {
    if link.is_empty() {
        return None;
    }
    let query = link.split_once('?').map(|(_, rest)| rest)?;
    let bare = query.split('#').next().unwrap_or(query);
    for pair in bare.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == "cursor" || key == "page%5Bcursor%5D" || key == "page[cursor]" {
                let decoded = urlencoding::decode(value).ok()?.into_owned();
                if !decoded.is_empty() {
                    return Some(decoded);
                }
            }
        }
    }
    None
}

// `ConnectorHealthResponse` and `NotificationChannelsResponse` envelope
// types are now sourced from `coauth-admin-types` (their wire shape was
// drifting — sodmin's `connectors:` envelope key did not match the
// backend's `providers:`).

#[cfg(test)]
mod cursor_tests {
    use super::{extract_cursor_param, AccountListFilter};

    #[test]
    fn extract_cursor_handles_plain_param() {
        assert_eq!(
            extract_cursor_param("/api/admin/v1/accounts?cursor=ABC123&limit=25"),
            Some("ABC123".to_string()),
        );
    }

    #[test]
    fn extract_cursor_handles_jsonapi_bracketed_param() {
        assert_eq!(
            extract_cursor_param("/api/admin/v1/accounts?page%5Bcursor%5D=DEF456"),
            Some("DEF456".to_string()),
        );
    }

    #[test]
    fn extract_cursor_returns_none_when_absent() {
        assert!(extract_cursor_param("/api/admin/v1/accounts?limit=25").is_none());
        assert!(extract_cursor_param("").is_none());
    }

    #[test]
    fn extract_cursor_decodes_url_escaping() {
        // base64url uses only [A-Za-z0-9_-], but wire data sometimes
        // includes URL-encoded `=` padding — make sure we round-trip the
        // raw cursor token cleanly.
        assert_eq!(
            extract_cursor_param("/x?cursor=A%3DB&limit=25"),
            Some("A=B".to_string()),
        );
    }

    #[test]
    fn account_list_filter_is_empty_when_blank() {
        let f = AccountListFilter::default();
        assert!(f.is_empty());
        let f = AccountListFilter {
            handle: "  ".to_string(),
            display_name: "".to_string(),
        };
        assert!(f.is_empty());
        let f = AccountListFilter {
            handle: "alice".to_string(),
            display_name: "".to_string(),
        };
        assert!(!f.is_empty());
    }
}
