//! HTTP client for the soland moderation reports admin surface
//!
//!
//! Endpoints:
//!
//! - `GET /_soland/admin/moderation/reports` — paginated list of open reports (default; `?status=…`
//!   widens the projection).
//! - `POST /_soland/admin/moderation/reports/{id}/resolve` — admin decision body `{decision:
//!   "resolve" | "dismiss", note?}`. Same 404-tolerant pattern as the rest of Stream H'.

use crate::api::client::{api_client, build_url};
use crate::api::generated::soland_admin::{ModerationReport, ResolveReportRequest};
use crate::utils::net::error::HttpError;

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
    let url = build_url("/_soland/admin/moderation/reports", &params)?;
    let resp: ModerationReportEnvelope = api_client(&url, "GET", None).await?;
    Ok(ModerationReportPage {
        data: resp.data,
        next_cursor: resp.next_cursor,
        total: resp.total,
    })
}

pub async fn resolve_report(report_id: &str, body: &ResolveReportRequest) -> Result<(), HttpError> {
    let url = format!(
        "/_soland/admin/moderation/reports/{}/resolve",
        urlencoding::encode(report_id)
    );
    let payload = serde_json::to_string(body).unwrap_or_default();
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}

// ── Moderation appeals ──────────────────────────────────────────────

/// Wire shape returned by `GET /_soland/admin/moderation/appeals`.
/// Each entry is one row per `appeal_id`, with the latest event of
/// that appeal (the soland helper collapses the event history to the
/// most recent state). Optional fields may be absent when the current
/// appeal state does not populate them.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct AppealRowDto {
    #[serde(default)]
    pub appeal_id: String,
    #[serde(default)]
    pub decision_ref: Option<String>,
    #[serde(default)]
    pub target_ref: Option<String>,
    #[serde(default)]
    pub appellant: Option<String>,
    #[serde(default)]
    pub reason_text_ref: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    /// `submitted` | `under_review` | `decided` | `closed`.
    #[serde(default)]
    pub appeal_state: Option<String>,
    #[serde(default)]
    pub original_issuer_did: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct AppealListEnvelope {
    #[serde(default)]
    items: Vec<AppealRowDto>,
}

pub async fn list_appeals() -> Result<Vec<AppealRowDto>, HttpError> {
    let url = "/_soland/admin/moderation/appeals".to_owned();
    let resp: AppealListEnvelope = api_client(&url, "GET", None).await?;
    Ok(resp.items)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DecideAppealRequest {
    /// `uphold` | `overturn` | `modify`.
    pub verdict: String,
    pub reason_text_ref: String,
    /// Required iff `verdict=modify`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modify_decision_ref: Option<String>,
    /// Required iff `verdict=overturn`. Caller MUST first POST a
    /// decision-lift via `decision_lift`, then thread the resulting
    /// lift event_id here.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_lift_ref: Option<String>,
}

pub async fn decide_appeal(
    appeal_id: &str,
    body: &DecideAppealRequest,
) -> Result<serde_json::Value, HttpError> {
    let url = format!(
        "/_soland/admin/moderation/appeals/{}/decision",
        urlencoding::encode(appeal_id)
    );
    let payload = serde_json::to_string(body).unwrap_or_default();
    api_client(&url, "POST", Some(payload)).await
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LiftDecisionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_text_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appeal_ref: Option<String>,
}

pub async fn lift_decision(
    decision_id: &str,
    body: &LiftDecisionRequest,
) -> Result<serde_json::Value, HttpError> {
    let url = format!(
        "/_soland/admin/moderation/decision/{}/lift",
        urlencoding::encode(decision_id)
    );
    let payload = serde_json::to_string(body).unwrap_or_default();
    api_client(&url, "POST", Some(payload)).await
}
