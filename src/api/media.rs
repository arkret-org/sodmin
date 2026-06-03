use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn get_media_statistics() -> Result<MediaStatistics, HttpError> {
    api_client("/_soland/admin/media/statistics", "GET", None).await
}

pub async fn list_actor_media(
    page: u64,
    per_page: u64,
) -> Result<ListResponse<ActorMediaStatistics>, HttpError> {
    let url = build_url(
        "/_soland/admin/media/by-actor",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}
