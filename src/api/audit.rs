use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

/// Multi-dimensional filter for `/admin/v1/audit` queries. Empty fields
/// are dropped before encoding so the wire form only carries what the
/// operator actually filtered on.
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub action: Option<String>,
    pub actor_id: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
}

impl AuditFilter {
    fn into_query(self) -> Vec<(&'static str, String)> {
        [
            ("action", self.action),
            ("actor_id", self.actor_id),
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

pub async fn list_audit_entries(
    page: u64,
    per_page: u64,
    filter: AuditFilter,
) -> Result<ListResponse<AuditEntry>, HttpError> {
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
    let url = build_url("/api/admin/v1/audit", &params)?;
    api_client(&url, "GET", None).await
}
