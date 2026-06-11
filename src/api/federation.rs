use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::net::error::HttpError;

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
    let url = build_url("/_soland/admin/federation", &params)?;
    let mut resp: ListResponse<serde_json::Value> = api_client(&url, "GET", None).await?;
    let needle = search.trim().to_ascii_lowercase();
    let data = resp
        .data
        .drain(..)
        .map(federation_snapshot_to_peer)
        .filter(|peer| {
            needle.is_empty()
                || peer.domain.to_ascii_lowercase().contains(&needle)
                || peer
                    .connection_id
                    .as_deref()
                    .is_some_and(|value| value.to_ascii_lowercase().contains(&needle))
        })
        .collect::<Vec<_>>();
    Ok(ListResponse {
        data,
        total: resp.total,
        next_cursor: resp.next_cursor,
    })
}

pub async fn get_federation_peer(domain: &str) -> Result<FederationPeer, HttpError> {
    let page = list_federation_peers(None, 10_000, "").await?;
    page.data
        .into_iter()
        .find(|peer| peer.domain == domain || peer.connection_id.as_deref() == Some(domain))
        .ok_or_else(|| HttpError::message("federation peer detail endpoint is not wired"))
}

pub async fn reset_federation_connection(_domain: &str) -> Result<(), HttpError> {
    Err(HttpError::message("federation reset endpoint is not wired"))
}

pub async fn list_federation_allow_rules(
    cursor: Option<&str>,
    limit: u64,
) -> Result<ListResponse<FederationAllowRule>, HttpError> {
    Ok(ListResponse {
        data: Vec::new(),
        total: Some(0),
        next_cursor: None,
    })
}

pub async fn add_federation_rule(
    _request: &AddFederationRuleRequest,
) -> Result<FederationAllowRule, HttpError> {
    Err(HttpError::message(
        "federation allow-rule write endpoint is not wired",
    ))
}

pub async fn defederate_federation_peer(_domain: &str) -> Result<(), HttpError> {
    Err(HttpError::message(
        "federation defederate endpoint is not wired",
    ))
}

pub async fn delete_federation_allow_rule(_id: &str) -> Result<(), HttpError> {
    Err(HttpError::message(
        "federation allow-rule delete endpoint is not wired",
    ))
}

fn federation_snapshot_to_peer(value: serde_json::Value) -> FederationPeer {
    let operation_id = string_field(&value, &["operation_id", "id"]);
    let realm_id = string_field(&value, &["realm_id"]);
    let operation_type = string_field(&value, &["operation_type", "canonical_kind"]);
    FederationPeer {
        domain: string_field(&value, &["service_did", "peer_did", "domain"])
            .or(operation_id.clone())
            .unwrap_or_else(|| "-".to_string()),
        status: operation_type.clone(),
        trust_level: None,
        last_successful_txn: string_field(&value, &["created_at"]),
        last_error: None,
        retry_interval: 0,
        direction: string_field(&value, &["direction"]).or_else(|| Some("operation".to_string())),
        connection_id: realm_id.or(operation_id),
    }
}

fn string_field(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    })
}
