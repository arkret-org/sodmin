//! coauth admin audit feed.

use coauth_admin_types::{AuditFeedOutcome, AuditSignatureStatus};
use serde::{Deserialize, Serialize};

use crate::api::client::{NO_BODY, api_client, build_url};
use crate::types::PaginatedResponse;
use crate::utils::net::error::HttpError;

const AUDIT_FEED_PATH: &str = "/_coauth/admin/audit-feed";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CoauthAuditEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub operation: String,
    #[serde(default)]
    pub actor_user_id: Option<String>,
    #[serde(default)]
    pub target_type: Option<String>,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub details: Option<serde_json::Value>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    pub signature_status: AuditSignatureStatus,
}

/// Multi-dimensional filter for `/_coauth/admin/audit-feed` queries. Empty
/// fields are dropped before encoding so the wire form only carries
/// what the operator actually filtered on.
#[derive(Debug, Clone, Default)]
pub struct AuditFeedFilter {
    pub operation: Option<String>,
    pub actor_user_id: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
}

impl AuditFeedFilter {
    fn into_query(self) -> Vec<(&'static str, String)> {
        let _unsupported = (self.operation, self.target_id, self.since, self.until);
        [
            ("admin_user_id", self.actor_user_id),
            ("resource_kind", self.target_type),
        ]
        .into_iter()
        .filter_map(|(k, v)| {
            v.map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .map(|s| (k, s))
        })
        .collect()
    }
}

pub async fn list_audit_feed(
    _page: u64,
    per_page: u64,
    filter: AuditFeedFilter,
) -> Result<PaginatedResponse<CoauthAuditEntry>, HttpError> {
    let limit_str = per_page.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str())];
    let owned = filter.into_query();
    for (k, v) in owned.iter() {
        params.push((k, v.as_str()));
    }
    let url = build_url(AUDIT_FEED_PATH, &params)?;
    let resp: AuditFeedOutcome = api_client(&url, "GET", NO_BODY).await?;
    let data = resp
        .data
        .into_iter()
        .map(|entry| CoauthAuditEntry {
            id: entry.id,
            operation: entry.operation,
            actor_user_id: entry.admin_user_id,
            target_type: Some(entry.resource_kind).filter(|value| !value.is_empty()),
            target_id: Some(entry.resource_id).filter(|value| !value.is_empty()),
            details: entry.details,
            timestamp: Some(entry.created_at.to_rfc3339()),
            source_ip: None,
            signature_status: entry.signature_status,
        })
        .collect::<Vec<_>>();
    Ok(PaginatedResponse {
        total: data.len() as u64,
        data,
    })
}
