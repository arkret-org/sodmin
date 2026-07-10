use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

use crate::api::client::{NO_BODY, api_client, build_url};
use crate::types::*;
use crate::utils::net::error::HttpError;

const FILTERED_AUDIT_FETCH_LIMIT: u64 = 1000;

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
    /// AKP-0007 (P3A.5) — server-side filter on event kind. When set,
    /// soland constrains the audit feed to the matching `ak.*` kind
    /// strings (e.g. `ak.circle.create`).
    pub event_kind: Option<String>,
    /// AKP-0007 (P3A.5) — server-side filter on effective scope (a
    /// `ak:realm:...` or `ak:circle:...` id).
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
    let filters = filter.into_query();
    if filters.is_empty() {
        return fetch_audit_page(Some(&cursor), limit).await;
    }
    list_filtered_audit_entries(page, limit, &filters).await
}

async fn fetch_audit_page(
    cursor: Option<&str>,
    limit: u64,
) -> Result<ListResponse<AuditEntry>, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str())];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url("/_soland/admin/audit", &params)?;
    api_client(&url, "GET", NO_BODY).await
}

async fn list_filtered_audit_entries(
    page: u64,
    per_page: u64,
    filters: &[(&'static str, String)],
) -> Result<ListResponse<AuditEntry>, HttpError> {
    let mut cursor = None::<String>;
    let mut matched = Vec::new();

    loop {
        let resp = fetch_audit_page(cursor.as_deref(), FILTERED_AUDIT_FETCH_LIMIT).await?;
        matched.extend(
            resp.data
                .into_iter()
                .filter(|entry| audit_entry_matches(entry, filters)),
        );
        match resp.next_cursor {
            Some(next) if !next.is_empty() && cursor.as_deref() != Some(next.as_str()) => {
                cursor = Some(next);
            }
            _ => break,
        }
    }

    let total = matched.len() as u64;
    let start = page.saturating_sub(1).saturating_mul(per_page) as usize;
    let data: Vec<AuditEntry> = matched
        .into_iter()
        .skip(start)
        .take(per_page as usize)
        .collect();
    let next_cursor = if (start as u64).saturating_add(data.len() as u64) < total {
        Some(page.saturating_add(1).to_string())
    } else {
        None
    };
    Ok(ListResponse {
        data,
        total: Some(total),
        next_cursor,
    })
}

fn audit_entry_matches(entry: &AuditEntry, filters: &[(&'static str, String)]) -> bool {
    filters.iter().all(|(key, value)| {
        if matches!(*key, "since" | "until") {
            return audit_timestamp_matches(entry.timestamp.as_deref(), key, value);
        }
        let needle = value.to_ascii_lowercase();
        let haystack = match *key {
            "action" | "event_kind" => entry.action.as_str(),
            "actor_id" => entry.actor_id.as_deref().unwrap_or_default(),
            "target_type" => entry.target_type.as_deref().unwrap_or_default(),
            "target_id" => entry.target_id.as_deref().unwrap_or_default(),
            "effective_scope" => entry.effective_scope.as_deref().unwrap_or_default(),
            _ => return true,
        };
        haystack.to_ascii_lowercase().contains(&needle)
    })
}

fn audit_timestamp_matches(timestamp: Option<&str>, key: &str, value: &str) -> bool {
    let Some(filter_at) = parse_audit_time(value) else {
        return true;
    };
    let Some(entry_at) = timestamp.and_then(parse_audit_time) else {
        return false;
    };
    match key {
        "since" => entry_at >= filter_at,
        "until" => entry_at <= filter_at,
        _ => true,
    }
}

fn parse_audit_time(value: &str) -> Option<DateTime<Utc>> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
                .ok()
                .map(|dt| Utc.from_utc_datetime(&dt))
        })
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
                .ok()
                .map(|dt| Utc.from_utc_datetime(&dt))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_at(timestamp: &str) -> AuditEntry {
        AuditEntry {
            action: "login".to_owned(),
            timestamp: Some(timestamp.to_owned()),
            ..Default::default()
        }
    }

    #[test]
    fn audit_date_filters_keep_entries_inside_window() {
        let entry = entry_at("2026-06-21T10:30:00Z");
        let filters = vec![
            ("since", "2026-06-21T10:00".to_owned()),
            ("until", "2026-06-21T11:00".to_owned()),
        ];
        assert!(audit_entry_matches(&entry, &filters));
    }

    #[test]
    fn audit_date_filters_reject_entries_outside_window() {
        let entry = entry_at("2026-06-21T09:59:59Z");
        let filters = vec![("since", "2026-06-21T10:00".to_owned())];
        assert!(!audit_entry_matches(&entry, &filters));

        let entry = entry_at("2026-06-21T11:00:01Z");
        let filters = vec![("until", "2026-06-21T11:00".to_owned())];
        assert!(!audit_entry_matches(&entry, &filters));
    }
}
