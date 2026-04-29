use serde::{Deserialize, Serialize};

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
        "/api/admin/v1/users",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
            ("search", search),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn get_user(id: &str) -> Result<CoauthUser, HttpError> {
    let url = format!("/api/admin/v1/users/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn update_user(id: &str, patch: &serde_json::Value) -> Result<CoauthUser, HttpError> {
    let url = format!("/api/admin/v1/users/{}", urlencoding::encode(id));
    api_client(&url, "PATCH", Some(patch.to_string())).await
}

pub async fn set_user_password(id: &str, password: &str) -> Result<(), HttpError> {
    let url = format!("/api/admin/v1/users/{}/set-password", urlencoding::encode(id));
    let body = serde_json::json!({ "password": password });
    api_client(&url, "POST", Some(body.to_string())).await
}

pub async fn list_audit_feed(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthAuditEntry>, HttpError> {
    let url = build_url(
        "/api/admin/v1/audit-feed",
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
        "/api/admin/v1/oauth2-sessions",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn finish_oauth2_session(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/admin/v1/oauth2-sessions/{}/finish", urlencoding::encode(id));
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
    api_client("/api/admin/v1/personal-sessions", "POST", Some(body.to_string())).await
}

pub async fn revoke_personal_session(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/admin/v1/personal-sessions/{}/revoke", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

pub async fn regenerate_personal_session(
    id: &str,
) -> Result<CoauthPersonalSession, HttpError> {
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
    let resp: ConnectorHealthResponse =
        api_client("/api/admin/v1/connector-health", "GET", None).await?;
    Ok(resp.connectors)
}

pub async fn list_notification_channels() -> Result<Vec<CoauthNotificationChannel>, HttpError> {
    let resp: NotificationChannelsResponse =
        api_client("/api/admin/v1/notification-channels", "GET", None).await?;
    Ok(resp.channels)
}

pub async fn list_notification_templates(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthNotificationTemplate>, HttpError> {
    let url = build_url(
        "/api/admin/v1/notification-templates",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn publish_notification_templates() -> Result<(), HttpError> {
    api_client(
        "/api/admin/v1/notification-templates/publish",
        "POST",
        None,
    )
    .await
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
