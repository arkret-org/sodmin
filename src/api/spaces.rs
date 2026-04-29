use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_spaces(
    page: u64,
    per_page: u64,
    search: &str,
) -> Result<ListResponse<Space>, HttpError> {
    let url = build_url(
        "/contrix/admin/v1/spaces",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
            ("search", search),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn get_space(id: &str) -> Result<Space, HttpError> {
    let url = format!("/contrix/admin/v1/spaces/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn create_space(req: &CreateSpaceRequest) -> Result<Space, HttpError> {
    api_client(
        "/contrix/admin/v1/spaces",
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn delete_space(id: &str) -> Result<(), HttpError> {
    let url = format!("/contrix/admin/v1/spaces/{}", urlencoding::encode(id));
    api_client(&url, "DELETE", None).await
}

pub async fn block_space(id: &str) -> Result<(), HttpError> {
    let url = format!("/contrix/admin/v1/spaces/{}/block", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

pub async fn unblock_space(id: &str) -> Result<(), HttpError> {
    let url = format!("/contrix/admin/v1/spaces/{}/unblock", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

pub async fn list_space_members(id: &str) -> Result<Vec<SpaceMember>, HttpError> {
    let url = format!("/contrix/admin/v1/spaces/{}/members", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn kick_space_member(space_id: &str, actor_id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/spaces/{}/members/{}/kick",
        urlencoding::encode(space_id),
        urlencoding::encode(actor_id)
    );
    api_client(&url, "POST", None).await
}

pub async fn ban_space_member(space_id: &str, actor_id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/contrix/admin/v1/spaces/{}/members/{}/ban",
        urlencoding::encode(space_id),
        urlencoding::encode(actor_id)
    );
    api_client(&url, "POST", None).await
}
