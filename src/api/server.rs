use crate::api::client::{NO_BODY, api_client};
use crate::types::*;
use crate::utils::net::error::HttpError;

const SERVER_INFO_PATH: &str = "/_soland/admin/server/info";
const SERVER_DESCRIBE_PATH: &str = "/_arkret/describe";
const SERVER_STATUS_PATH: &str = "/_soland/admin/server/status";
const SERVER_STATS_PATH: &str = "/_soland/admin/server/stats";

pub async fn get_server_info() -> Result<ServerInfo, HttpError> {
    api_client(SERVER_INFO_PATH, "GET", NO_BODY).await
}

pub async fn get_server_describe() -> Result<ServerDescribeDocument, HttpError> {
    api_client(SERVER_DESCRIBE_PATH, "GET", NO_BODY).await
}

pub async fn get_coauth_server_describe() -> Result<ServerDescribeDocument, HttpError> {
    let url = crate::utils::net::session::coauth_public_url()
        .map(|base| format!("{}/_arkret/describe", base.trim_end_matches('/')))
        .unwrap_or_else(|| "/_arkret/describe".to_string());
    api_client(&url, "GET", NO_BODY).await
}

pub async fn get_server_stats() -> Result<ServerStats, HttpError> {
    api_client(SERVER_STATS_PATH, "GET", NO_BODY).await
}

pub async fn get_server_status() -> Result<ServerStatusOutcome, HttpError> {
    api_client(SERVER_STATUS_PATH, "GET", NO_BODY).await
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DestroyRealmRequest {
    pub confirmation: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RealmDeleteOutcome {
    pub realm_id: String,
    pub deleted: bool,
}

pub async fn destroy_realm(
    realm_id: &str,
    _body: &DestroyRealmRequest,
) -> Result<RealmDeleteOutcome, HttpError> {
    let url = format!("/_soland/admin/realms/{}", urlencoding::encode(realm_id));
    api_client(&url, "DELETE", NO_BODY).await
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
    api_client("/health", "GET", NO_BODY).await
}

/// T8.3 — fetch coauth `/health` against the configured upstream URL.
pub async fn get_coauth_health() -> Result<HealthEnvelope, HttpError> {
    let url = crate::utils::net::session::coauth_public_url()
        .map(|base| format!("{}/health", base.trim_end_matches('/')))
        .unwrap_or_else(|| "/health".to_string());
    api_client(&url, "GET", NO_BODY).await
}

/// T8.3 — fetch starid `/health` against the configured upstream URL.
pub async fn get_starid_health() -> Result<HealthEnvelope, HttpError> {
    let url = crate::utils::net::session::starid_public_url()
        .map(|base| format!("{}/health", base.trim_end_matches('/')))
        .unwrap_or_else(|| "/health".to_string());
    api_client(&url, "GET", NO_BODY).await
}
