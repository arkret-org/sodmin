use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

/// Multi-dimensional filter for `/admin/audit` queries. Empty fields
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
    /// CXP-0007 (P3A.5) — server-side filter on event kind. When set,
    /// soland constrains the audit feed to the matching `cx.*` kind
    /// strings (e.g. `cx.circle.create`).
    pub event_kind: Option<String>,
    /// CXP-0007 (P3A.5) — server-side filter on effective scope (a
    /// `ck:realm:...` or `ck:circle:...` id).
    pub effective_scope: Option<String>,
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
            ("event_kind", self.event_kind),
            ("effective_scope", self.effective_scope),
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
    let url = build_url("/_soland/admin/audit", &params)?;
    api_client(&url, "GET", None).await
}
