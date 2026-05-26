use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

/// Cursor-paginated space list. `search` is a best-effort
/// `filter[name_or_id]` parameter; backends that haven't shipped it
/// just return everything and the page filters client-side.
pub async fn list_spaces(
    cursor: Option<&str>,
    limit: u64,
    search: &str,
) -> Result<ListResponse<Space>, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str())];
    if let Some(c) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", c));
    }
    if !search.is_empty() {
        params.push(("filter[name_or_id]", search));
    }
    let url = build_url("/api/admin/v1/spaces", &params)?;
    api_client(&url, "GET", None).await
}

pub async fn get_space(id: &str) -> Result<Space, HttpError> {
    let url = format!("/api/admin/v1/spaces/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn create_space(req: &CreateSpaceRequest) -> Result<Space, HttpError> {
    api_client(
        "/api/admin/v1/spaces",
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn delete_space(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/admin/v1/spaces/{}", urlencoding::encode(id));
    api_client(&url, "DELETE", None).await
}

pub async fn block_space(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/admin/v1/spaces/{}/block", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

pub async fn unblock_space(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/admin/v1/spaces/{}/unblock", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

pub async fn list_space_members(id: &str) -> Result<Vec<SpaceMember>, HttpError> {
    let url = format!("/api/admin/v1/spaces/{}/members", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn kick_space_member(space_id: &str, actor_id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/spaces/{}/members/{}/kick",
        urlencoding::encode(space_id),
        urlencoding::encode(actor_id)
    );
    api_client(&url, "POST", None).await
}

pub async fn ban_space_member(space_id: &str, actor_id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/spaces/{}/members/{}/ban",
        urlencoding::encode(space_id),
        urlencoding::encode(actor_id)
    );
    api_client(&url, "POST", None).await
}
