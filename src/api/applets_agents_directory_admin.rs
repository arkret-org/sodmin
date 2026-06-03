//! HTTP client for the soland applets/agents/directory admin surfaces.
//!
//! All routes are 404-tolerant — soland may not have these admin
//! surfaces wired yet for every deployment. The page surfaces a
//! "endpoint not yet wired" toast on 404.

use coauth_admin_types::applets_admin::{
    AgentAdminRow, AppletAdminRow, ApprovalActionRequest, DirectoryAdminRow,
};

use crate::api::client::{api_client, build_url};
use crate::utils::error::HttpError;

#[derive(Debug, Clone, Default)]
pub struct AppletAdminPage {
    pub data: Vec<AppletAdminRow>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct AppletAdminEnvelope {
    #[serde(default)]
    data: Vec<AppletAdminRow>,
    #[serde(default)]
    next_cursor: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AgentAdminPage {
    pub data: Vec<AgentAdminRow>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct AgentAdminEnvelope {
    #[serde(default)]
    data: Vec<AgentAdminRow>,
    #[serde(default)]
    next_cursor: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DirectoryAdminPage {
    pub data: Vec<DirectoryAdminRow>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct DirectoryAdminEnvelope {
    #[serde(default)]
    data: Vec<DirectoryAdminRow>,
    #[serde(default)]
    next_cursor: Option<String>,
}

fn cursor_params<'a>(cursor: Option<&'a str>, limit_str: &'a str) -> Vec<(&'a str, &'a str)> {
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str)];
    if let Some(c) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", c));
    }
    params
}

// ── Applets ───────────────────────────────────────────────────────────

pub async fn list_applets(cursor: Option<&str>, limit: u64) -> Result<AppletAdminPage, HttpError> {
    let limit_str = limit.max(1).to_string();
    let params = cursor_params(cursor, limit_str.as_str());
    let url = build_url("/_soland/admin/applets", &params)?;
    let resp: AppletAdminEnvelope = api_client(&url, "GET", None).await?;
    Ok(AppletAdminPage {
        data: resp.data,
        next_cursor: resp.next_cursor,
    })
}

pub async fn approve_applet(id: &str, body: &ApprovalActionRequest) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/applets/{}/approve", urlencoding::encode(id));
    let payload = serde_json::to_string(body).unwrap_or_default();
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}

pub async fn suspend_applet(id: &str, body: &ApprovalActionRequest) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/applets/{}/suspend", urlencoding::encode(id));
    let payload = serde_json::to_string(body).unwrap_or_default();
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}

pub async fn revoke_applet(id: &str, body: &ApprovalActionRequest) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/applets/{}/revoke", urlencoding::encode(id));
    let payload = serde_json::to_string(body).unwrap_or_default();
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}

// ── Agents ────────────────────────────────────────────────────────────

pub async fn list_agents_admin(
    cursor: Option<&str>,
    limit: u64,
) -> Result<AgentAdminPage, HttpError> {
    let limit_str = limit.max(1).to_string();
    let params = cursor_params(cursor, limit_str.as_str());
    let url = build_url("/_soland/admin/agents", &params)?;
    let resp: AgentAdminEnvelope = api_client(&url, "GET", None).await?;
    Ok(AgentAdminPage {
        data: resp.data,
        next_cursor: resp.next_cursor,
    })
}

pub async fn approve_agent(id: &str, body: &ApprovalActionRequest) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/agents/{}/approve", urlencoding::encode(id));
    let payload = serde_json::to_string(body).unwrap_or_default();
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}

pub async fn suspend_agent(id: &str, body: &ApprovalActionRequest) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/agents/{}/suspend", urlencoding::encode(id));
    let payload = serde_json::to_string(body).unwrap_or_default();
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}

pub async fn revoke_agent(id: &str, body: &ApprovalActionRequest) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/agents/{}/revoke", urlencoding::encode(id));
    let payload = serde_json::to_string(body).unwrap_or_default();
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}

// ── Directory ─────────────────────────────────────────────────────────

pub async fn list_directory_admin(
    cursor: Option<&str>,
    limit: u64,
) -> Result<DirectoryAdminPage, HttpError> {
    let limit_str = limit.max(1).to_string();
    let params = cursor_params(cursor, limit_str.as_str());
    let url = build_url("/api/admin/v1/directory", &params)?;
    let resp: DirectoryAdminEnvelope = api_client(&url, "GET", None).await?;
    Ok(DirectoryAdminPage {
        data: resp.data,
        next_cursor: resp.next_cursor,
    })
}

pub async fn approve_directory_entry(
    id: &str,
    body: &ApprovalActionRequest,
) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/directory/{}/approve",
        urlencoding::encode(id)
    );
    let payload = serde_json::to_string(body).unwrap_or_default();
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}

/// F3 directory uses "reject" semantics on a Pending entry — same wire
/// shape as the agents/applets `/revoke` endpoint but soland keys on the
/// path so directory entries can keep a separate audit category.
pub async fn reject_directory_entry(
    id: &str,
    body: &ApprovalActionRequest,
) -> Result<(), HttpError> {
    let url = format!("/api/admin/v1/directory/{}/reject", urlencoding::encode(id));
    let payload = serde_json::to_string(body).unwrap_or_default();
    let _: serde_json::Value = api_client(&url, "POST", Some(payload)).await?;
    Ok(())
}
