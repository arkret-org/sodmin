//! Compatibility wrapper over the soland admin capability snapshot.
//!
//! The old authz capability write surface never existed. Keep the read
//! helper for callers that still want the authz row shape, but back it
//! with the admin capabilities snapshot and no direct grant/revoke writes.

use crate::api::capabilities;
use crate::api::contracts::soland_admin::{AuthzCapabilityGrant, AuthzGrantFilter};
use crate::types::CapabilityGrantExt;
use crate::utils::net::error::HttpError;

/// Cursor-shaped page wrapper for the authz admin surface. Matches the
/// JSON:API-ish envelope soland emits for paginated admin endpoints.
#[derive(Debug, Clone, Default)]
pub struct AuthzGrantPage {
    pub data: Vec<AuthzCapabilityGrant>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

/// Fetch a page of capability grants. `cursor` is the opaque server-issued
/// cursor; pass `None` for the first page.
pub async fn list_capability_grants(
    cursor: Option<&str>,
    limit: u64,
    filter: &AuthzGrantFilter,
) -> Result<AuthzGrantPage, HttpError> {
    let page_num = cursor
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1);
    let resp = capabilities::list_capabilities(page_num, limit).await?;
    let mut data = resp
        .data
        .iter()
        .map(|grant| AuthzCapabilityGrant {
            grant_id: grant.id.to_string(),
            holder_did: grant.subject_display(),
            peer_did: grant.issuer.to_string(),
            scope: grant.resources_display(),
            status: if grant.is_revoked() {
                "revoked".to_string()
            } else {
                "active".to_string()
            },
            granted_at: Some(grant.issued_at.to_rfc3339()),
            expires_at: grant.expires_at.map(|value| value.to_rfc3339()),
            note: None,
        })
        .collect::<Vec<_>>();
    if !filter.is_empty() {
        data = crate::api::contracts::soland_admin::filter_grants(&data, filter);
    }
    Ok(AuthzGrantPage {
        data,
        next_cursor: resp.next_cursor,
        total: resp.total,
    })
}
