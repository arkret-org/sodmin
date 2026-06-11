use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::net::error::HttpError;

pub async fn list_media(page: u64, per_page: u64) -> Result<ListResponse<MediaRow>, HttpError> {
    let limit = per_page.max(1);
    let cursor = page.saturating_sub(1).saturating_mul(limit).to_string();
    let limit_str = limit.to_string();
    let url = build_url(
        "/_soland/admin/media",
        &[("limit", limit_str.as_str()), ("cursor", &cursor)],
    )?;
    api_client(&url, "GET", None).await
}
