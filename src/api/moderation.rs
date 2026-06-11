//! HTTP client for the soland moderation reports admin surface
//!
//!
//! Endpoints:
//!
//! - `GET /_soland/admin/reports` — admin snapshot of submitted reports.
//! - `GET /_soland/admin/moderation/queue` — canonical moderation queue snapshot.
//! - `POST /_soland/admin/moderation/decision` — issue the closing moderation decision.

use crate::api::client::{api_client, build_url, json_body};
use crate::api::contracts::soland_admin::{ModerationReport, ReportDecision, ResolveReportRequest};
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
    reports: Vec<ModerationReport>,
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    total: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct ModerationQueueEnvelope {
    #[serde(default)]
    items: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct IssueDecisionRequest {
    target_ref: String,
    realm_id: String,
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason_text_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    queue_item_ref: Option<String>,
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
    let url = build_url("/_soland/admin/reports", &params)?;
    let resp: ModerationReportEnvelope = api_client(&url, "GET", None).await?;
    let mut reports = if resp.data.is_empty() {
        resp.reports
    } else {
        resp.data
    };
    if let Ok(queue) = list_queue_reports().await {
        reports.extend(queue);
    }
    if status != "all" {
        reports.retain(|report| report.status == status);
    }
    Ok(ModerationReportPage {
        data: reports,
        next_cursor: resp.next_cursor,
        total: resp.total,
    })
}

async fn list_queue_reports() -> Result<Vec<ModerationReport>, HttpError> {
    let resp: ModerationQueueEnvelope =
        api_client("/_soland/admin/moderation/queue", "GET", None).await?;
    Ok(resp
        .items
        .into_iter()
        .map(queue_item_to_report)
        .collect::<Vec<_>>())
}

pub async fn resolve_report(
    report: &ModerationReport,
    body: &ResolveReportRequest,
) -> Result<(), HttpError> {
    let payload = json_body(&IssueDecisionRequest {
        target_ref: report
            .target_ref
            .clone()
            .unwrap_or_else(|| report.report_id.clone()),
        realm_id: report.realm_id.clone().unwrap_or_else(|| "*".to_string()),
        action: match body.decision {
            ReportDecision::Resolve => "resolve".to_string(),
            ReportDecision::Dismiss => "dismiss".to_string(),
        },
        reason_text_ref: body.note.clone(),
        queue_item_ref: Some(report.report_id.clone()),
    })?;
    let url = "/_soland/admin/moderation/decision";
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}

fn queue_item_to_report(item: serde_json::Value) -> ModerationReport {
    let id = first_string(&item, &["id", "queue_item_id", "report_id"]).unwrap_or_default();
    let target_ref = first_string(&item, &["target_ref", "target", "subject_ref"]);
    let realm_id = first_string(&item, &["realm_id", "scope"]);
    let reason = first_string(&item, &["reason", "reason_text", "summary"]).unwrap_or_default();
    let created_at = first_string(&item, &["created_at", "submitted_at", "updated_at"]);
    ModerationReport {
        report_id: id,
        reporter_did: first_string(&item, &["reporter_did", "reporter", "submitted_by"])
            .unwrap_or_default(),
        target_ref,
        realm_id,
        reason,
        status: first_string(&item, &["status"]).unwrap_or_else(|| "open".to_string()),
        created_at,
        note: None,
    }
}

fn first_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    })
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
    let payload = json_body(body)?;
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
    let payload = json_body(body)?;
    api_client(&url, "POST", Some(payload)).await
}
