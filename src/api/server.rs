use crate::api::client::api_client;
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn get_server_info() -> Result<ServerInfo, HttpError> {
    api_client("/contrix/admin/v1/server/info", "GET", None).await
}

pub async fn get_server_describe() -> Result<ServerDescribeResponse, HttpError> {
    api_client("/api/v1/server/describe", "GET", None).await
}

pub async fn get_coauth_server_describe() -> Result<ServerDescribeResponse, HttpError> {
    let url = crate::utils::session::coauth_public_url()
        .map(|base| format!("{}/api/v1/server/describe", base.trim_end_matches('/')))
        .unwrap_or_else(|| "/api/v1/server/describe".to_string());
    api_client(&url, "GET", None).await
}

pub async fn get_server_stats() -> Result<ServerStats, HttpError> {
    api_client("/contrix/admin/v1/server/stats", "GET", None).await
}

pub async fn get_server_status() -> Result<ServerStatusResponse, HttpError> {
    api_client("/contrix/admin/v1/server/status", "GET", None).await
}
