//! HTTP client for the soland Space container admin surface.
//!
//! Endpoints:
//!
//! - `GET /_soland/admin/spaces` — list of Spaces visible to the current admin scope.
//!   Cursor-paginated.
//! Per-space hierarchy views are assembled client-side from the same
//! snapshot; soland does not expose a dedicated hierarchy endpoint.

use crate::api::client::{api_client, build_url};
use crate::types::api::ListResponse;
use crate::types::spaces::{SpaceHierarchy, SpaceHierarchyNode, SpaceRow};
use crate::utils::net::error::HttpError;

#[derive(Debug, Clone, Default)]
pub struct SpacePage {
    pub data: Vec<SpaceRow>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
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
    let resp: ListResponse<SpaceRow> = api_client(&url, "GET", None).await?;
    let needle = search.trim().to_ascii_lowercase();
    let data = if needle.is_empty() {
        resp.data
    } else {
        resp.data
            .into_iter()
            .filter(|row| {
                row.id.to_ascii_lowercase().contains(&needle)
                    || row
                        .name
                        .as_deref()
                        .is_some_and(|name| name.to_ascii_lowercase().contains(&needle))
            })
            .collect()
    };
    Ok(SpacePage {
        data,
        next_cursor: resp.next_cursor,
        total: resp.total,
    })
}

pub async fn get_space(space_id: &str) -> Result<SpaceHierarchy, HttpError> {
    let page = list_spaces(None, 10_000, "").await?;
    let Some(center) = page.data.iter().find(|row| row.id == space_id).cloned() else {
        return Err(HttpError::message("space not found in admin snapshot"));
    };
    let parent = center
        .parent_space_id
        .as_ref()
        .and_then(|parent_id| page.data.iter().find(|row| row.id == *parent_id))
        .map(space_node);
    let children = page
        .data
        .iter()
        .filter(|row| row.parent_space_id.as_deref() == Some(space_id))
        .map(space_node)
        .collect();
    Ok(SpaceHierarchy {
        space_id: center.id,
        name: center.name,
        parent,
        children,
    })
}

fn space_node(row: &SpaceRow) -> SpaceHierarchyNode {
    SpaceHierarchyNode {
        space_id: row.id.clone(),
        name: row.name.clone(),
        member_count: row.member_count,
    }
}
