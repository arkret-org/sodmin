use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_capabilities(
    page: u64,
    per_page: u64,
) -> Result<ListResponse<CapabilityGrant>, HttpError> {
    let url = build_url(
        "/api/admin/v1/capabilities",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn grant_capability(req: &GrantCapabilityRequest) -> Result<CapabilityGrant, HttpError> {
    api_client(
        "/api/admin/v1/capabilities",
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn revoke_capability(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/capabilities/{}/revoke",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

/// PATCH `/api/admin/v1/capabilities/{id}` — fine-grained constraint
/// edits (T6.2 §5). Sends only the changed fields; the server merges
/// them into the existing grant and publishes a Move.
pub async fn update_capability(
    id: &str,
    req: &UpdateCapabilityRequest,
) -> Result<CapabilityGrant, HttpError> {
    let url = format!("/api/admin/v1/capabilities/{}", urlencoding::encode(id));
    api_client(
        &url,
        "PATCH",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}
