use crate::api::client::api_client;
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn get_server_info() -> Result<ServerInfo, HttpError> {
    api_client("/api/admin/v1/server/info", "GET", None).await
}

pub async fn get_server_describe() -> Result<ServerDescribeResBody, HttpError> {
    api_client("/api/v1/server/describe", "GET", None).await
}

pub async fn get_coauth_server_describe() -> Result<ServerDescribeResBody, HttpError> {
    let url = crate::utils::session::coauth_public_url()
        .map(|base| format!("{}/api/v1/server/describe", base.trim_end_matches('/')))
        .unwrap_or_else(|| "/api/v1/server/describe".to_string());
    api_client(&url, "GET", None).await
}

pub async fn get_server_stats() -> Result<ServerStats, HttpError> {
    api_client("/api/admin/v1/server/stats", "GET", None).await
}

pub async fn get_server_status() -> Result<ServerStatusResponse, HttpError> {
    api_client("/api/admin/v1/server/status", "GET", None).await
}

/// T8.3 — `/health` envelope deserialized for the hardening dashboard.
/// Each service exposes a `hardening` block (see `HardeningStatus`).
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct HealthEnvelope {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub service: Option<String>,
    #[serde(default)]
    pub development_mode: Option<bool>,
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
    let url = crate::utils::session::coauth_public_url()
        .map(|base| format!("{}/health", base.trim_end_matches('/')))
        .unwrap_or_else(|| "/health".to_string());
    api_client(&url, "GET", None).await
}

/// T8.3 — fetch starid `/health` against the configured upstream URL.
pub async fn get_starid_health() -> Result<HealthEnvelope, HttpError> {
    let url = crate::utils::session::starid_public_url()
        .map(|base| format!("{}/health", base.trim_end_matches('/')))
        .unwrap_or_else(|| "/health".to_string());
    api_client(&url, "GET", None).await
}
