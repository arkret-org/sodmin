use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_actors(
    page: u64,
    per_page: u64,
    search: &str,
) -> Result<ListResponse<Actor>, HttpError> {
    let url = build_url(
        "/api/admin/v1/actors",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
            ("search", search),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn get_actor(id: &str) -> Result<Actor, HttpError> {
    let url = format!("/api/admin/v1/actors/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn create_actor(req: &CreateActorRequest) -> Result<Actor, HttpError> {
    api_client(
        "/api/admin/v1/actors",
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn update_actor(id: &str, req: &UpdateActorRequest) -> Result<Actor, HttpError> {
    let url = format!("/api/admin/v1/actors/{}", urlencoding::encode(id));
    api_client(
        &url,
        "PATCH",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn deactivate_actor(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/actors/{}/deactivate",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn check_handle_availability(
    handle: &str,
) -> Result<HandleAvailabilityResult, HttpError> {
    let url = build_url(
        "/api/admin/v1/actors/handle-availability",
        &[("handle", handle)],
    )?;
    api_client(&url, "GET", None).await
}
