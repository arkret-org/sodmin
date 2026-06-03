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
) -> Result<ListResponse<Realm>, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str())];
    if let Some(c) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", c));
    }
    if !search.is_empty() {
        params.push(("filter[name_or_id]", search));
    }
    let url = build_url("/admin/spaces", &params)?;
    api_client(&url, "GET", None).await
}

pub async fn get_space(id: &str) -> Result<Realm, HttpError> {
    let url = format!("/admin/spaces/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn create_space(req: &CreateRealmRequest) -> Result<Realm, HttpError> {
    api_client(
        "/admin/spaces",
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn delete_space(id: &str) -> Result<(), HttpError> {
    let url = format!("/admin/spaces/{}", urlencoding::encode(id));
    api_client(&url, "DELETE", None).await
}

pub async fn block_space(id: &str) -> Result<(), HttpError> {
    let url = format!("/admin/spaces/{}/block", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

pub async fn unblock_space(id: &str) -> Result<(), HttpError> {
    let url = format!("/admin/spaces/{}/unblock", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

pub async fn list_space_members(id: &str) -> Result<Vec<SpaceMember>, HttpError> {
    let url = format!("/admin/spaces/{}/members", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}
