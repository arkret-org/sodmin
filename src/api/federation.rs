use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_federation_peers(
    page: u64,
    per_page: u64,
) -> Result<ListResponse<FederationPeer>, HttpError> {
    let url = build_url(
        "/_cx/admin/v1/federation/peers",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn get_federation_peer(domain: &str) -> Result<FederationPeer, HttpError> {
    let url = format!(
        "/_cx/admin/v1/federation/peers/{}",
        urlencoding::encode(domain)
    );
    api_client(&url, "GET", None).await
}

pub async fn reset_federation_connection(domain: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_cx/admin/v1/federation/peers/{}/reset",
        urlencoding::encode(domain)
    );
    api_client(&url, "POST", None).await
}

pub async fn list_federation_allow_rules(
    page: u64,
    per_page: u64,
) -> Result<ListResponse<FederationAllowRule>, HttpError> {
    let url = build_url(
        "/_cx/admin/v1/federation/allow-rules",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn add_federation_allow_rule(domain: &str) -> Result<FederationAllowRule, HttpError> {
    let body = serde_json::json!({ "domain": domain });
    api_client(
        "/_cx/admin/v1/federation/allow-rules",
        "POST",
        Some(body.to_string()),
    )
    .await
}

pub async fn delete_federation_allow_rule(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_cx/admin/v1/federation/allow-rules/{}",
        urlencoding::encode(id)
    );
    api_client(&url, "DELETE", None).await
}
