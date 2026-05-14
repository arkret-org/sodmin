//! HTTP client for the soland moderation reports admin surface
//!
//!
//! Endpoints:
//!
//! - `GET /api/admin/v1/moderation/reports` — paginated list of open
//!   reports (default; `?status=…` widens the projection).
//! - `POST /api/admin/v1/moderation/reports/{id}/resolve` — admin
//!   decision body `{decision: "resolve" | "dismiss", note?}`. Same
//!   404-tolerant pattern as the rest of Stream H'.

use crate::api::client::{api_client, build_url};
use crate::types::moderation::{ModerationReport, ResolveReportRequest};
use crate::utils::error::HttpError;

#[derive(Debug, Clone, Default)]
pub struct ModerationReportPage {
    pub data: Vec<ModerationReport>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct ModerationReportEnvelope {
    #[serde(default)]
    data: Vec<ModerationReport>,
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    total: Option<u64>,
}

pub async fn list_reports(
    cursor: Option<&str>,
    limit: u64,
    status: &str,
) -> Result<ModerationReportPage, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> =
        vec![("status", status.trim()), ("limit", limit_str.as_str())];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url("/api/admin/v1/moderation/reports", &params)?;
    let resp: ModerationReportEnvelope = api_client(&url, "GET", None).await?;
    Ok(ModerationReportPage {
        data: resp.data,
        next_cursor: resp.next_cursor,
        total: resp.total,
    })
}

pub async fn resolve_report(report_id: &str, body: &ResolveReportRequest) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/moderation/reports/{}/resolve",
        urlencoding::encode(report_id)
    );
    let payload = serde_json::to_string(body).unwrap_or_default();
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}
