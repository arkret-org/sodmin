use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn get_media_statistics() -> Result<MediaStatistics, HttpError> {
    api_client("/_cx/admin/v1/media/statistics", "GET", None).await
}

pub async fn list_actor_media(
    page: u64,
    per_page: u64,
) -> Result<ListResponse<ActorMediaStatistics>, HttpError> {
    let url = build_url(
        "/_cx/admin/v1/media/by-actor",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn list_blobs(
    page: u64,
    per_page: u64,
    actor_id: &str,
) -> Result<ListResponse<BlobInfo>, HttpError> {
    let url = build_url(
        "/_cx/admin/v1/media/blobs",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
            ("actor_id", actor_id),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn quarantine_blob(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_cx/admin/v1/media/blobs/{}/quarantine",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn delete_blob(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_cx/admin/v1/media/blobs/{}",
        urlencoding::encode(id)
    );
    api_client(&url, "DELETE", None).await
}

pub async fn purge_remote_media(before_ts: Option<u64>) -> Result<serde_json::Value, HttpError> {
    let url = build_url(
        "/_cx/admin/v1/media/purge-remote",
        &[("before_ts", &before_ts.map(|t| t.to_string()).unwrap_or_default())],
    )?;
    api_client(&url, "POST", None).await
}
