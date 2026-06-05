use crate::api::client::{api_client, build_url};
use crate::api::generated::soland_admin::{CreatePolicyRequest, Policy, PolicyListResponse};
use crate::utils::net::error::HttpError;

pub async fn list_policies(
    cursor: Option<&str>,
    limit: u64,
) -> Result<PolicyListResponse, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str())];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url("/_soland/admin/policies", &params)?;
    api_client(&url, "GET", None).await
}

pub async fn create_policy(req: &CreatePolicyRequest) -> Result<Policy, HttpError> {
    api_client(
        "/_soland/admin/policies",
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn update_policy(id: &str, req: &CreatePolicyRequest) -> Result<Policy, HttpError> {
    let url = format!("/_soland/admin/policies/{}", urlencoding::encode(id));
    api_client(
        &url,
        "PUT",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn delete_policy(id: &str) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/policies/{}", urlencoding::encode(id));
    api_client(&url, "DELETE", None).await
}
