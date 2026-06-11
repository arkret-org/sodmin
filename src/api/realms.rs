use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::net::error::HttpError;

/// Cursor-paginated Realm list.
pub async fn list_realms(
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
        params.push(("filter[search]", search));
    }
    let url = build_url("/_soland/admin/realms", &params)?;
    api_client(&url, "GET", None).await
}

pub async fn get_realm(id: &str) -> Result<Realm, HttpError> {
    let url = format!("/_soland/admin/realms/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn create_realm(req: &CreateRealmRequest) -> Result<Realm, HttpError> {
    api_client(
        "/_soland/admin/realms",
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn list_realm_members(id: &str) -> Result<Vec<RealmMember>, HttpError> {
    let url = format!("/_soland/admin/realms/{}/members", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}
