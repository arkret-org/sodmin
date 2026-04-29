use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn get_server_info() -> Result<ServerInfo, HttpError> {
    api_client("/_cx/admin/v1/server/info", "GET", None).await
}

pub async fn get_server_stats() -> Result<ServerStats, HttpError> {
    api_client("/_cx/admin/v1/server/stats", "GET", None).await
}

pub async fn get_server_status() -> Result<ServerStatusResponse, HttpError> {
    api_client("/_cx/admin/v1/server/status", "GET", None).await
}
