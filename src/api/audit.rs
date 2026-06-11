use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::net::error::HttpError;

/// Multi-dimensional filter for `/_soland/admin/audit` queries. Empty fields
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
    /// CKP-0007 (P3A.5) — server-side filter on event kind. When set,
    /// soland constrains the audit feed to the matching `ck.*` kind
    /// strings (e.g. `ck.circle.create`).
    pub event_kind: Option<String>,
    /// CKP-0007 (P3A.5) — server-side filter on effective scope (a
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
    let limit = per_page.max(1);
    let cursor = page.saturating_sub(1).saturating_mul(limit).to_string();
    let limit_str = limit.to_string();
    let params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str()), ("cursor", &cursor)];
    let url = build_url("/_soland/admin/audit", &params)?;
    let mut resp: ListResponse<AuditEntry> = api_client(&url, "GET", None).await?;
    let filters = filter.into_query();
    if !filters.is_empty() {
        resp.data
            .retain(|entry| audit_entry_matches(entry, &filters));
    }
    Ok(resp)
}

fn audit_entry_matches(entry: &AuditEntry, filters: &[(&'static str, String)]) -> bool {
    filters.iter().all(|(key, value)| {
        let needle = value.to_ascii_lowercase();
        let haystack = match *key {
            "action" | "event_kind" => entry.action.as_str(),
            "actor_id" => entry.actor_id.as_deref().unwrap_or_default(),
            "target_type" => entry.target_type.as_deref().unwrap_or_default(),
            "target_id" => entry.target_id.as_deref().unwrap_or_default(),
            "effective_scope" => entry.effective_scope.as_deref().unwrap_or_default(),
            _ => "",
        };
        haystack.to_ascii_lowercase().contains(&needle)
    })
}
