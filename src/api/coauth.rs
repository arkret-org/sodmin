//! coauth admin API client surface.
//!
//! Wire shapes that already live in the shared `coauth-admin-types`
//! crate are imported / re-exported from there. Anything still defined
//! inline here is on the migration list — when the corresponding coauth
//! admin handler graduates to a typed response, lift the struct into
//! `coauth-admin-types` and turn the local copy into a re-export.

use serde::{Deserialize, Serialize};

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

pub use coauth_admin_types::AdminBridgeDescribe as CoauthAdminBridgeDescribe;
pub use coauth_admin_types::IntegrationManifest as CoauthIntegrationManifest;

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
    pub integration_manifest: CoauthIntegrationManifest,
}

pub use coauth_admin_types::AccountRiskActionApprovalRequest as CoauthAccountRiskActionApprovalDraft;
pub use coauth_admin_types::AccountRiskActionApprovalResponse as CoauthAccountRiskActionApproval;
pub use coauth_admin_types::AccountRiskActionCurrentResponse as CoauthAccountRiskActionCurrentState;
pub use coauth_admin_types::AccountRiskActionExecuteRequest as CoauthAccountRiskActionExecuteDraft;
pub use coauth_admin_types::AccountRiskActionHistoryResponse as CoauthAccountRiskActionHistoryEnvelopeShared;
pub use coauth_admin_types::AccountRiskActionProposalRequest as CoauthAccountRiskActionDraft;
pub use coauth_admin_types::AccountRiskActionProposalResponse as CoauthAccountRiskActionProposal;
pub use coauth_admin_types::AccountRiskActionTransitionRecord as CoauthAccountRiskActionHistoryEntry;

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

type CoauthAccountClaimsEnvelope = coauth_admin_types::AdminAccountClaimsResponse;

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

type CoauthAdminSingleEnvelope<T> = coauth_admin_types::SingleResponse<T>;
type CoauthAdminResource<T> = coauth_admin_types::SingleResource<T>;

#[derive(Debug, Clone, Deserialize, Default)]
struct CoauthAdminPaginationMeta {
    #[serde(default)]
    count: Option<u64>,
}

type CoauthAdminAccountRecord = coauth_admin_types::AdminAccountAttributes;
type CoauthAdminDidBindingsEnvelope = coauth_admin_types::AdminAccountDidBindingsResponse;
type CoauthAdminDidBindingRecord = coauth_admin_types::AdminAccountDidBinding;

// ── API methods ──

pub async fn get_viewer() -> Result<CoauthViewer, HttpError> {
    api_client("/api/v1/viewer", "GET", None).await
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
    let url = build_url("/api/admin/v1/audit-feed", &params)?;
    api_client(&url, "GET", None).await
}

pub async fn list_oauth2_sessions(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthOAuth2Session>, HttpError> {
    let url = build_url(
        "/api/admin/v1/oauth2-sessions",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn finish_oauth2_session(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/oauth2-sessions/{}/finish",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn list_personal_sessions(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthPersonalSession>, HttpError> {
    let url = build_url(
        "/api/admin/v1/personal-sessions",
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
        "/api/admin/v1/personal-sessions",
        "POST",
        Some(body.to_string()),
    )
    .await
}

pub async fn revoke_personal_session(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/personal-sessions/{}/revoke",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn regenerate_personal_session(id: &str) -> Result<CoauthPersonalSession, HttpError> {
    let url = format!(
        "/api/admin/v1/personal-sessions/{}/regenerate",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn list_upstream_providers(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthUpstreamProvider>, HttpError> {
    let url = build_url(
        "/api/admin/v1/upstream-oauth-providers",
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
        "/api/admin/v1/upstream-oauth-providers",
        "POST",
        Some(provider.to_string()),
    )
    .await
}

pub async fn delete_upstream_provider(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/upstream-oauth-providers/{}",
        urlencoding::encode(id)
    );
    api_client(&url, "DELETE", None).await
}

pub async fn toggle_upstream_provider(id: &str, enable: bool) -> Result<(), HttpError> {
    let action = if enable { "enable" } else { "disable" };
    let url = format!(
        "/api/admin/v1/upstream-oauth-providers/{}/{}",
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
        "/api/admin/v1/upstream-oauth-links",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn delete_upstream_link(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/upstream-oauth-links/{}",
        urlencoding::encode(id)
    );
    api_client(&url, "DELETE", None).await
}

pub async fn list_registration_tokens(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthRegistrationToken>, HttpError> {
    let url = build_url(
        "/api/admin/v1/user-registration-tokens",
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
        "/api/admin/v1/user-registration-tokens",
        "POST",
        Some(body.to_string()),
    )
    .await
}

pub async fn revoke_registration_token(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/user-registration-tokens/{}/revoke",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn get_connector_health() -> Result<Vec<CoauthConnectorHealth>, HttpError> {
    let resp: coauth_admin_types::ConnectorHealthResponse =
        api_client("/api/admin/v1/connector-health", "GET", None).await?;
    Ok(resp.providers)
}

pub async fn list_notification_channels() -> Result<Vec<CoauthNotificationChannel>, HttpError> {
    let resp: coauth_admin_types::NotificationChannelsResponse =
        api_client("/api/admin/v1/notification-channels", "GET", None).await?;
    Ok(resp.channels)
}

/// Returns the static catalog of known notification template keys. The
/// backend endpoint is non-paginated — it returns the full known-keys
/// list every call — so the sodmin UI now flat-lists the entries
/// instead of pretending there's a paging cursor.
pub async fn list_notification_templates() -> Result<Vec<CoauthNotificationTemplate>, HttpError> {
    let resp: coauth_admin_types::NotificationTemplatesResponse =
        api_client("/api/admin/v1/notification-templates", "GET", None).await?;
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
        "/api/admin/v1/notification-templates/publish",
        "POST",
        Some(body),
    )
    .await
}

/// Cursor-paginated account list. The caller passes back the opaque
/// `cursor` it received from the prior page's `links.next`.
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
    let url = build_url("/api/admin/v1/accounts", &params)?;
    let resp: CoauthAdminPaginatedEnvelope<CoauthAdminAccountRecord> =
        api_client(&url, "GET", None).await?;
    let summaries: Vec<CoauthAccountSummary> = resp
        .data
        .unwrap_or_default()
        .into_iter()
        .map(map_admin_account_summary_resource)
        .collect();
    let next_cursor = resp.links.next.as_deref().and_then(extract_cursor_param);
    let prev_cursor = resp.links.prev.as_deref().and_then(extract_cursor_param);
    Ok(CursorPage {
        data: summaries,
        next_cursor,
        prev_cursor,
        total: resp.meta.count,
    })
}

pub async fn get_account_detail(id: &str) -> Result<CoauthAccountDetail, HttpError> {
    let bridge_url = "/api/admin/v1/bridge/describe";
    let integration_manifest_url = "/contrix/api/v1/integration/describe";
    let summary_url = format!("/api/admin/v1/accounts/{}", urlencoding::encode(id));
    let dids_url = format!(
        "/api/admin/v1/accounts/{}/dids",
        urlencoding::encode(id)
    );
    let claims_url = format!(
        "/api/admin/v1/accounts/{}/claims",
        urlencoding::encode(id)
    );
    let grants_url = format!(
        "/api/admin/v1/accounts/{}/session-grants",
        urlencoding::encode(id)
    );
    let current_url = format!(
        "/api/admin/v1/accounts/{}/risk-action/current",
        urlencoding::encode(id)
    );
    let history_url = format!(
        "/api/admin/v1/accounts/{}/risk-action/history",
        urlencoding::encode(id)
    );
    let summary: CoauthAdminSingleEnvelope<CoauthAdminAccountRecord> =
        api_client(&summary_url, "GET", None).await?;
    let dids: CoauthAdminDidBindingsEnvelope = api_client(&dids_url, "GET", None).await?;
    let claims: CoauthAccountClaimsEnvelope = api_client(&claims_url, "GET", None).await?;
    let session_grants: CoauthAccountSessionGrantsEnvelope =
        api_client(&grants_url, "GET", None).await?;
    let bridge: CoauthAdminBridgeDescribe = api_client(bridge_url, "GET", None).await?;
    let integration_manifest: CoauthIntegrationManifest =
        api_client(integration_manifest_url, "GET", None).await?;
    let current: CoauthAdminSingleEnvelope<CoauthAccountRiskActionCurrentState> =
        api_client(&current_url, "GET", None).await?;
    let history: CoauthAccountRiskActionHistoryEnvelopeShared =
        api_client(&history_url, "GET", None).await?;
    let account = map_admin_account_summary_resource(summary.data);
    Ok(CoauthAccountDetail {
        claims: claims
            .data
            .into_iter()
            .map(map_admin_account_claim)
            .collect(),
        session_grants: session_grants.data,
        risk_action_current: current.data.attributes,
        risk_action_history: history.data,
        managed_dids: dids.data.into_iter().map(map_admin_did_binding).collect(),
        account,
        risk_action_hook: CoauthRiskActionHook {
            endpoint: bridge.risk_action_path_template.replace("{account_id}", id),
            approval_mode: bridge.risk_action_approval_mode.clone(),
            todo: if bridge.todos.is_empty() {
                "Coauth account admin bridge advertises no outstanding operator follow-ups."
                    .to_string()
            } else {
                bridge.todos.join(" ")
            },
        },
        admin_bridge: bridge,
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
        "/api/admin/v1/accounts/{}/dids",
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
pub async fn remove_account_did_binding(account_id: &str, did: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/accounts/{}/dids/{}",
        urlencoding::encode(account_id),
        urlencoding::encode(did),
    );
    let _: serde_json::Value = api_client(&url, "DELETE", None).await?;
    Ok(())
}

/// Revoke a single claim attached to the account. The claim is keyed by
/// its `claim_type` (e.g. `email`, `principal_did`); coauth's claims
/// admin routes accept the type as a path segment.
pub async fn revoke_account_claim(account_id: &str, claim_type: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/accounts/{}/claims/{}/revoke",
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
        "/api/admin/v1/accounts/{}/risk-action",
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
        "/api/admin/v1/accounts/{}/risk-action/{}/approve",
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
        "/api/admin/v1/accounts/{}/risk-action/{}/execute",
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
    let url = format!(
        "/api/admin/v1/accounts/{}/lock",
        urlencoding::encode(id)
    );
    let _: serde_json::Value = api_client(&url, "POST", None).await?;
    Ok(())
}

pub async fn disable_account(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/accounts/{}/disable",
        urlencoding::encode(id)
    );
    let _: serde_json::Value = api_client(&url, "POST", None).await?;
    Ok(())
}

pub async fn erase_account(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/accounts/{}/erase",
        urlencoding::encode(id)
    );
    let _: serde_json::Value = api_client(&url, "POST", None).await?;
    Ok(())
}

pub async fn reset_account_recovery(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/accounts/{}/reset-recovery",
        urlencoding::encode(id)
    );
    let _: serde_json::Value = api_client(&url, "POST", None).await?;
    Ok(())
}

fn map_admin_account_summary_resource(
    resource: CoauthAdminResource<CoauthAdminAccountRecord>,
) -> CoauthAccountSummary {
    let attributes = resource.attributes;
    let is_locked = attributes.status.is_locked();
    let is_deactivated = attributes.status.is_disabled();
    let primary_did = attributes.effective_primary_did().map(str::to_owned);
    let bridge_status = if attributes.admin {
        "coauth_admin_accounts_v1+admin".to_string()
    } else {
        "coauth_admin_accounts_v1".to_string()
    };
    CoauthAccountSummary {
        id: resource.id,
        username: Some(attributes.handle),
        display_name: attributes.display_name,
        avatar_url: attributes.avatar_url,
        email: None,
        primary_did,
        is_locked,
        is_deactivated,
        created_at: attributes.created_at.map(|t| t.to_rfc3339()),
        updated_at: attributes.updated_at.map(|t| t.to_rfc3339()),
        bridge_status,
    }
}

fn map_admin_did_binding(binding: CoauthAdminDidBindingRecord) -> CoauthManagedDidBinding {
    use coauth_admin_types::{DidBindingKind, DidBindingState, DidBindingVerificationStatus};
    let kind_wire = match binding.kind {
        DidBindingKind::Primary => "primary",
        DidBindingKind::Recovery => "recovery",
        DidBindingKind::Pairwise => "pairwise",
    };
    let state_wire = match binding.state {
        DidBindingState::PendingProof => "pending_proof",
        DidBindingState::Active => "active",
        DidBindingState::Revoked => "revoked",
        DidBindingState::Rejected => "rejected",
    };
    let verification_wire = match binding.verification_status {
        DidBindingVerificationStatus::Pending => "pending",
        DidBindingVerificationStatus::Verified => "verified",
        DidBindingVerificationStatus::Rejected => "rejected",
        DidBindingVerificationStatus::NotRequested => "not_requested",
    };
    CoauthManagedDidBinding {
        did: binding.did,
        method: Some(kind_wire.to_owned()),
        state: Some(format!(
            "{}:{}:{}",
            state_wire,
            verification_wire,
            if binding.primary {
                "primary"
            } else {
                "secondary"
            }
        )),
        last_verified_at: binding.last_verified_at.map(|t| t.to_rfc3339()),
    }
}

fn map_admin_account_claim(
    record: coauth_admin_types::AdminAccountClaimRecord,
) -> CoauthAccountClaim {
    CoauthAccountClaim {
        claim_type: record.claim_type,
        value: record.value,
        state: Some(record.state),
        source: Some(record.source),
    }
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

#[cfg(test)]
mod cursor_tests {
    use super::{AccountListFilter, extract_cursor_param};

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
