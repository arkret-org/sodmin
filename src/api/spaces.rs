//! HTTP client for the soland Space container admin surface.
//!
//! Endpoints:
//!
//! - `GET /_soland/admin/spaces` — list of Spaces visible to the current admin scope.
//!   Cursor-paginated.
//! - `GET /_soland/admin/spaces/{id}/hierarchy` — parent + immediate children for a single Space.
//!
//! Both routes are 404-tolerant on the client side — the
//! `format_optional_endpoint_error` helper turns 404 into a clear
//! "not yet wired" toast so operators can tell "feature not deployed"
//! apart from "row missing".

use crate::api::client::{api_client, build_url};
use crate::types::spaces::{SpaceHierarchy, SpaceRow};
use crate::utils::net::error::HttpError;

#[derive(Debug, Clone, Default)]
pub struct SpacePage {
    pub data: Vec<SpaceRow>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct SpaceEnvelope {
    #[serde(default)]
    data: Vec<SpaceRow>,
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    total: Option<u64>,
}

pub async fn list_spaces(
    cursor: Option<&str>,
    limit: u64,
    search: &str,
) -> Result<SpacePage, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![
        ("filter[search]", search.trim()),
        ("limit", limit_str.as_str()),
    ];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url("/_soland/admin/spaces", &params)?;
    let resp: SpaceEnvelope = api_client(&url, "GET", None).await?;
    Ok(SpacePage {
        data: resp.data,
        next_cursor: resp.next_cursor,
        total: resp.total,
    })
}

pub async fn get_space(space_id: &str) -> Result<SpaceHierarchy, HttpError> {
    let url = format!(
        "/_soland/admin/spaces/{}/hierarchy",
        urlencoding::encode(space_id)
    );
    api_client(&url, "GET", None).await
}
