//! HTTP client for the soland recovery / restore-ticket admin surface
//! (Round 25, C1-C5).
//!
//! All routes follow the 404-tolerant pattern shared with the rest of
//! Stream H' — when the backend hasn't wired the surface yet, the page
//! surfaces a clear "endpoint not yet wired" toast via
//! `format_optional_endpoint_error`.

use coauth_admin_types::recovery_admin::{
    RecoveryActionRequest, RecoveryAuditEntry, RecoveryDescribe, RecoveryTicket,
    RecoveryTicketDetail,
};

use crate::api::client::{api_client, build_url};
use crate::utils::error::HttpError;

#[derive(Debug, Clone, Default)]
pub struct RecoveryTicketPage {
    pub data: Vec<RecoveryTicket>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct RecoveryTicketEnvelope {
    #[serde(default)]
    data: Vec<RecoveryTicket>,
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    total: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct RecoveryAuditPage {
    pub data: Vec<RecoveryAuditEntry>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct RecoveryAuditEnvelope {
    #[serde(default)]
    data: Vec<RecoveryAuditEntry>,
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    total: Option<u64>,
}

/// `GET /api/admin/v1/coauth/recovery-tickets?status=&cursor=&limit=`.
pub async fn list_tickets(
    cursor: Option<&str>,
    limit: u64,
    status: &str,
) -> Result<RecoveryTicketPage, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![
        ("status", status.trim()),
        ("limit", limit_str.as_str()),
    ];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url("/api/admin/v1/coauth/recovery-tickets", &params)?;
    let resp: RecoveryTicketEnvelope = api_client(&url, "GET", None).await?;
    Ok(RecoveryTicketPage {
        data: resp.data,
        next_cursor: resp.next_cursor,
        total: resp.total,
    })
}

/// `GET /api/admin/v1/coauth/recovery-tickets/{id}`.
pub async fn get_ticket(ticket_id: &str) -> Result<RecoveryTicketDetail, HttpError> {
    let url = format!(
        "/api/admin/v1/coauth/recovery-tickets/{}",
        urlencoding::encode(ticket_id)
    );
    api_client(&url, "GET", None).await
}

/// `POST /api/admin/v1/coauth/recovery-tickets/{id}/{action}`.
async fn post_action(
    ticket_id: &str,
    action: &str,
    body: &RecoveryActionRequest,
) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/coauth/recovery-tickets/{}/{}",
        urlencoding::encode(ticket_id),
        action,
    );
    let payload = serde_json::to_string(body).unwrap_or_default();
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}

pub async fn approve(ticket_id: &str, body: &RecoveryActionRequest) -> Result<(), HttpError> {
    post_action(ticket_id, "approve", body).await
}

pub async fn reject(ticket_id: &str, body: &RecoveryActionRequest) -> Result<(), HttpError> {
    post_action(ticket_id, "reject", body).await
}

pub async fn advance(ticket_id: &str, body: &RecoveryActionRequest) -> Result<(), HttpError> {
    post_action(ticket_id, "advance", body).await
}

pub async fn cancel(ticket_id: &str, body: &RecoveryActionRequest) -> Result<(), HttpError> {
    post_action(ticket_id, "cancel", body).await
}

/// `GET /api/admin/v1/coauth/recovery/audit`.
pub async fn list_audit(
    cursor: Option<&str>,
    limit: u64,
    ticket_id: &str,
) -> Result<RecoveryAuditPage, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![
        ("ticket_id", ticket_id.trim()),
        ("limit", limit_str.as_str()),
    ];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url("/api/admin/v1/coauth/recovery/audit", &params)?;
    let resp: RecoveryAuditEnvelope = api_client(&url, "GET", None).await?;
    Ok(RecoveryAuditPage {
        data: resp.data,
        next_cursor: resp.next_cursor,
        total: resp.total,
    })
}

/// `GET /api/admin/v1/coauth/recovery/describe`.
pub async fn describe() -> Result<RecoveryDescribe, HttpError> {
    api_client("/api/admin/v1/coauth/recovery/describe", "GET", None).await
}
