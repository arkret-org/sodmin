use crate::api::client::{api_client, build_url};
use crate::api::generated::soland_admin::{CreatePolicyRequest, Policy, PolicyListResponse};
use crate::utils::error::HttpError;

pub async fn list_policies(page: u64, per_page: u64) -> Result<PolicyListResponse, HttpError> {
    let url = build_url(
        "/api/admin/v1/policies",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn get_policy(id: &str) -> Result<Policy, HttpError> {
    let url = format!("/api/admin/v1/policies/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn create_policy(req: &CreatePolicyRequest) -> Result<Policy, HttpError> {
    api_client(
        "/api/admin/v1/policies",
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn update_policy(id: &str, req: &CreatePolicyRequest) -> Result<Policy, HttpError> {
    let url = format!("/api/admin/v1/policies/{}", urlencoding::encode(id));
    api_client(
        &url,
        "PUT",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn delete_policy(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/admin/v1/policies/{}", urlencoding::encode(id));
    api_client(&url, "DELETE", None).await
}
