use crate::api::client::api_client;
use crate::api::paths::soland as soland_paths;
use crate::types::*;
use crate::utils::net::error::HttpError;

pub async fn get_server_info() -> Result<ServerInfo, HttpError> {
    api_client(soland_paths::SERVER_INFO, "GET", None).await
}

pub async fn get_server_describe() -> Result<ServerDescribeDocument, HttpError> {
    api_client(soland_paths::SERVER_DESCRIBE, "GET", None).await
}

pub async fn get_coauth_server_describe() -> Result<ServerDescribeDocument, HttpError> {
    let url = crate::utils::net::session::coauth_public_url()
        .map(|base| format!("{}/_cokret/describe", base.trim_end_matches('/')))
        .unwrap_or_else(|| "/_cokret/describe".to_string());
    api_client(&url, "GET", None).await
}

pub async fn get_server_stats() -> Result<ServerStats, HttpError> {
    api_client(soland_paths::SERVER_STATS, "GET", None).await
}

pub async fn get_server_status() -> Result<ServerStatusOutcome, HttpError> {
    api_client(soland_paths::SERVER_STATUS, "GET", None).await
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DestroyRealmRequest {
    pub confirmation: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RetryRealmDestroyRequest {
    pub domain: String,
}

pub async fn destroy_realm(
    realm_id: &str,
    _body: &DestroyRealmRequest,
) -> Result<serde_json::Value, HttpError> {
    let url = format!("/_soland/admin/realms/{}", urlencoding::encode(realm_id));
    api_client(&url, "DELETE", None).await
}

pub async fn retry_realm_destroy(
    realm_id: &str,
    _body: &RetryRealmDestroyRequest,
) -> Result<serde_json::Value, HttpError> {
    let url = format!("/_soland/admin/realms/{}", urlencoding::encode(realm_id));
    api_client(&url, "DELETE", None).await
}

/// T8.3 — `/health` envelope deserialized for the hardening dashboard.
/// Each service exposes a `hardening` block (see `HardeningStatus`).
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct HealthEnvelope {
    #[serde(default)]
    pub hardening: Option<HardeningStatus>,
}

/// T8.3 — fetch principal-server (soland) `/health` for the hardening
/// dashboard. Returns the parsed envelope including the `hardening`
/// block; older soland builds without the field still parse cleanly
/// (Option::None).
pub async fn get_soland_health() -> Result<HealthEnvelope, HttpError> {
    api_client("/health", "GET", None).await
}

/// T8.3 — fetch coauth `/health` against the configured upstream URL.
pub async fn get_coauth_health() -> Result<HealthEnvelope, HttpError> {
    let url = crate::utils::net::session::coauth_public_url()
        .map(|base| format!("{}/health", base.trim_end_matches('/')))
        .unwrap_or_else(|| "/health".to_string());
    api_client(&url, "GET", None).await
}

/// T8.3 — fetch starid `/health` against the configured upstream URL.
pub async fn get_starid_health() -> Result<HealthEnvelope, HttpError> {
    let url = crate::utils::net::session::starid_public_url()
        .map(|base| format!("{}/health", base.trim_end_matches('/')))
        .unwrap_or_else(|| "/health".to_string());
    api_client(&url, "GET", None).await
}
