use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::client::{api_client, build_url};
use crate::utils::error::HttpError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoauthConnectorHealth {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoauthNotificationChannel {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub channel_type: Option<String>,
    #[serde(default)]
    pub is_healthy: bool,
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoauthNotificationTemplate {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub channel_type: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
pub struct CoauthRiskActionHook {
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub approval_mode: String,
    #[serde(default)]
    pub todo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
pub struct CoauthAdminBridgeRiskActionExamples {
    #[serde(default)]
    pub proposal_request: Value,
    #[serde(default)]
    pub approve_request: Value,
    #[serde(default)]
    pub execute_request: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub key_backup_rest_base: String,
    #[serde(default)]
    pub key_backup_schema: String,
    #[serde(default)]
    pub device_message_schema: String,
    #[serde(default)]
    pub principal_recovery_contract_stack_path: String,
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
    pub principal_restore_start_path: String,
    #[serde(default)]
    pub principal_restore_describe_path: String,
    #[serde(default)]
    pub principal_restore_ticket_path: String,
    #[serde(default)]
    pub principal_restore_ticket_advance_path: String,
    #[serde(default)]
    pub principal_restore_approval_status_path: String,
    #[serde(default)]
    pub principal_restore_approval_submit_path: String,
    #[serde(default)]
    pub principal_restore_executor_status_path: String,
    #[serde(default)]
    pub principal_restore_executor_enqueue_path: String,
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
pub struct CoauthAccountRiskActionCurrentState {
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub state_record_id: Option<String>,
    #[serde(default)]
    pub proposal_id: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub lifecycle_state: String,
    #[serde(default)]
    pub last_operation: Option<String>,
    #[serde(default)]
    pub transition_kind: Option<String>,
    #[serde(default)]
    pub previous_state: Option<String>,
    #[serde(default)]
    pub state_revision: Option<u64>,
    #[serde(default)]
    pub allowed_next_transitions: Vec<String>,
    #[serde(default)]
    pub ticket: Option<String>,
    #[serde(default)]
    pub recorded_at: Option<String>,
    #[serde(default)]
    pub recorded_by: Option<String>,
    #[serde(default)]
    pub recorded_by_username: Option<String>,
    #[serde(default)]
    pub execution_endpoint: Option<String>,
    #[serde(default)]
    pub mutation_endpoint: Option<String>,
    #[serde(default)]
    pub state_store_kind: String,
    #[serde(default)]
    pub todo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoauthAccountRiskActionDraft {
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub ticket: Option<String>,
    #[serde(default)]
    pub approved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoauthAccountRiskActionApprovalDraft {
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub ticket: Option<String>,
    #[serde(default)]
    pub approved_by: Option<String>,
    #[serde(default)]
    pub approval_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoauthAccountRiskActionExecuteDraft {
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub ticket: Option<String>,
    #[serde(default)]
    pub execution_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoauthAccountRiskActionProposal {
    #[serde(default)]
    pub state_record_id: String,
    #[serde(default)]
    pub proposal_id: String,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub ticket: Option<String>,
    #[serde(default)]
    pub approved_by: Option<String>,
    #[serde(default)]
    pub requested_at: Option<String>,
    #[serde(default)]
    pub requested_by: Option<String>,
    #[serde(default)]
    pub requested_by_username: Option<String>,
    #[serde(default)]
    pub previous_state: String,
    #[serde(default)]
    pub proposal_state: String,
    #[serde(default)]
    pub state_revision: u64,
    #[serde(default)]
    pub transition_kind: String,
    #[serde(default)]
    pub approval_mode: String,
    #[serde(default)]
    pub allowed_next_transitions: Vec<String>,
    #[serde(default)]
    pub execution_endpoint: String,
    #[serde(default)]
    pub state_store_kind: String,
    #[serde(default)]
    pub todo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoauthAccountRiskActionApproval {
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
    pub approval_state: String,
    #[serde(default)]
    pub state_revision: u64,
    #[serde(default)]
    pub transition_kind: String,
    #[serde(default)]
    pub approved_at: Option<String>,
    #[serde(default)]
    pub approved_by: Option<String>,
    #[serde(default)]
    pub approved_by_username: Option<String>,
    #[serde(default)]
    pub approval_note: Option<String>,
    #[serde(default)]
    pub execution_endpoint: String,
    #[serde(default)]
    pub allowed_next_transitions: Vec<String>,
    #[serde(default)]
    pub state_store_kind: String,
    #[serde(default)]
    pub todo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoauthAccountRiskActionHistoryEntry {
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub state_record_id: Option<String>,
    #[serde(default)]
    pub proposal_id: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub transition_kind: String,
    #[serde(default)]
    pub previous_state: Option<String>,
    #[serde(default)]
    pub next_state: String,
    #[serde(default)]
    pub state_revision: Option<u64>,
    #[serde(default)]
    pub ticket: Option<String>,
    #[serde(default)]
    pub recorded_at: Option<String>,
    #[serde(default)]
    pub recorded_by: Option<String>,
    #[serde(default)]
    pub recorded_by_username: Option<String>,
    #[serde(default)]
    pub execution_endpoint: Option<String>,
    #[serde(default)]
    pub mutation_endpoint: Option<String>,
    #[serde(default)]
    pub approval_note: Option<String>,
    #[serde(default)]
    pub execution_note: Option<String>,
    #[serde(default)]
    pub state_store_kind: String,
    #[serde(default)]
    pub todo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CoauthAccountRiskActionHistoryEnvelope {
    #[serde(default)]
    pub data: Vec<CoauthAccountRiskActionHistoryEntry>,
}

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
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAdminSingleEnvelope<T> {
    data: CoauthAdminResource<T>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAdminResource<T> {
    #[serde(default)]
    id: String,
    #[serde(default)]
    attributes: T,
}

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

pub async fn list_audit_feed(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthAuditEntry>, HttpError> {
    let url = build_url(
        "/contrix/admin/v1/audit-feed",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
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
    let resp: ConnectorHealthResponse =
        api_client("/contrix/admin/v1/connector-health", "GET", None).await?;
    Ok(resp.connectors)
}

pub async fn list_notification_channels() -> Result<Vec<CoauthNotificationChannel>, HttpError> {
    let resp: NotificationChannelsResponse =
        api_client("/contrix/admin/v1/notification-channels", "GET", None).await?;
    Ok(resp.channels)
}

pub async fn list_notification_templates(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthNotificationTemplate>, HttpError> {
    let url = build_url(
        "/contrix/admin/v1/notification-templates",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn publish_notification_templates() -> Result<(), HttpError> {
    api_client(
        "/contrix/admin/v1/notification-templates/publish",
        "POST",
        None,
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
    let history: CoauthAccountRiskActionHistoryEnvelope =
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

#[derive(Debug, Clone, Deserialize, Default)]
struct ConnectorHealthResponse {
    #[serde(default)]
    connectors: Vec<CoauthConnectorHealth>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NotificationChannelsResponse {
    #[serde(default)]
    channels: Vec<CoauthNotificationChannel>,
}
