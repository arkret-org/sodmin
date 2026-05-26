use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

/// Cursor-paginated federation peer list. `search` is a best-effort
/// `filter[name_or_id]` (i.e. domain) parameter; backends that haven't
/// shipped it just return everything and the page filters client-side.
pub async fn list_federation_peers(
    cursor: Option<&str>,
    limit: u64,
    search: &str,
) -> Result<ListResponse<FederationPeer>, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str())];
    if let Some(c) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", c));
    }
    if !search.is_empty() {
        params.push(("filter[name_or_id]", search));
    }
    let url = build_url("/api/admin/v1/federation/peers", &params)?;
    api_client(&url, "GET", None).await
}

pub async fn get_federation_peer(domain: &str) -> Result<FederationPeer, HttpError> {
    let url = format!(
        "/api/admin/v1/federation/peers/{}",
        urlencoding::encode(domain)
    );
    api_client(&url, "GET", None).await
}

pub async fn reset_federation_connection(domain: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/federation/peers/{}/reset",
        urlencoding::encode(domain)
    );
    api_client(&url, "POST", None).await
}

pub async fn list_federation_allow_rules(
    cursor: Option<&str>,
    limit: u64,
) -> Result<ListResponse<FederationAllowRule>, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str())];
    if let Some(c) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", c));
    }
    let url = build_url("/api/admin/v1/federation/allow-rules", &params)?;
    api_client(&url, "GET", None).await
}

pub async fn add_federation_allow_rule(domain: &str) -> Result<FederationAllowRule, HttpError> {
    let body = serde_json::json!({ "domain": domain });
    api_client(
        "/api/admin/v1/federation/allow-rules",
        "POST",
        Some(body.to_string()),
    )
    .await
}

pub async fn delete_federation_allow_rule(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/federation/allow-rules/{}",
        urlencoding::encode(id)
    );
    api_client(&url, "DELETE", None).await
}
