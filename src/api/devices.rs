use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_devices(
    page: u64,
    per_page: u64,
    actor_id: &str,
) -> Result<ListResponse<Device>, HttpError> {
    let url = build_url(
        "/api/admin/v1/devices",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
            ("actor_id", actor_id),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn delete_device(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/admin/v1/devices/{}", urlencoding::encode(id));
    api_client(&url, "DELETE", None).await
}
