//! coauth admin audit feed.

use serde::{Deserialize, Serialize};

use crate::api::client::{api_client, build_url};
use crate::api::openapi_contract::coauth as coauth_paths;
use crate::types::PaginatedResponse;
use crate::utils::net::error::HttpError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
}

/// Multi-dimensional filter for `/_soland/admin/audit-feed` queries. Empty
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
        [
            ("operation", self.operation),
            ("actor_user_id", self.actor_user_id),
            ("target_type", self.target_type),
            ("target_id", self.target_id),
            ("since", self.since),
            ("until", self.until),
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
    page: u64,
    per_page: u64,
    filter: AuditFeedFilter,
) -> Result<PaginatedResponse<CoauthAuditEntry>, HttpError> {
    let page_str = page.to_string();
    let per_page_str = per_page.to_string();
    let mut params: Vec<(&str, &str)> = vec![
        ("page", page_str.as_str()),
        ("per_page", per_page_str.as_str()),
    ];
    let owned = filter.into_query();
    for (k, v) in owned.iter() {
        params.push((k, v.as_str()));
    }
    let url = build_url(coauth_paths::AUDIT_FEED, &params)?;
    api_client(&url, "GET", None).await
}
