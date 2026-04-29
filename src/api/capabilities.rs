use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_capabilities(
    page: u64,
    per_page: u64,
) -> Result<ListResponse<CapabilityGrant>, HttpError> {
    let url = build_url(
        "/_cx/admin/v1/capabilities",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn grant_capability(req: &GrantCapabilityRequest) -> Result<CapabilityGrant, HttpError> {
    api_client(
        "/_cx/admin/v1/capabilities",
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn revoke_capability(id: &str) -> Result<(), HttpError> {
    let url = format!("/_cx/admin/v1/capabilities/{}/revoke", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}
