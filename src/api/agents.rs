use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

fn json_body<T: serde::Serialize>(value: &T) -> Result<String, HttpError> {
    serde_json::to_string(value).map_err(|e| HttpError {
        message: format!("serialize: {e}"),
        status: 0,
        body: None,
        request_id: None,
        retry_after_ms: None,
    })
}

pub async fn list_agents(page: u64, per_page: u64) -> Result<ListResponse<Agent>, HttpError> {
    let url = build_url(
        "/api/admin/v1/agents",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn get_agent(id: &str) -> Result<Agent, HttpError> {
    let url = format!("/api/admin/v1/agents/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

// ── CXP-0008 personal-agent surface (soland P2 aa76b91) ──
//
// The 11 new soland endpoints. UI form payloads are TODO(P3-impl)
// stubs; the wire calls below MUST be exercised so the contract is
// asserted from the admin SPA.

/// `GET /api/v1/agents` — list controller-self native personal agents.
pub async fn list_personal_agents(
    page: u64,
    per_page: u64,
) -> Result<ListResponse<Agent>, HttpError> {
    let url = build_url(
        "/api/v1/agents",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

/// `GET /api/v1/agents/{id}` — controller-self detail.
pub async fn get_personal_agent(id: &str) -> Result<Agent, HttpError> {
    let url = format!("/api/v1/agents/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

/// `POST /api/v1/agents` — `cx.agent.provision`. Step-3 of the wizard
/// completes via the coauth `accountability_grant` call below.
pub async fn provision_personal_agent(
    req: &AgentProvisionRequest,
) -> Result<AgentProvisionResponse, HttpError> {
    let body = json_body(req)?;
    api_client("/api/v1/agents", "POST", Some(body)).await
}

/// `POST /api/v1/agents/{id}/pause` — `cx.agent.pause`.
pub async fn pause_personal_agent(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/v1/agents/{}/pause", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

/// `POST /api/v1/agents/{id}/resume` — `cx.agent.resume`.
pub async fn resume_personal_agent(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/v1/agents/{}/resume", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

/// `POST /api/v1/agents/{id}/deactivate` — `cx.agent.deactivate`.
/// Destructive: callers MUST gate this through `ConfirmDialog` with
/// typed-keyword confirmation (`DEACTIVATE`).
pub async fn deactivate_personal_agent(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/v1/agents/{}/deactivate", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

/// `POST /api/v1/agents/{id}/rotate-key` — `cx.agent.rotate_key`.
pub async fn rotate_personal_agent_key(id: &str) -> Result<AgentProvisionResponse, HttpError> {
    let url = format!("/api/v1/agents/{}/rotate-key", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

/// `POST /api/v1/agents/{id}/grants` — `cx.agent.grant.attach`.
/// `action` is one of the 14 personal-agent capability actions; the
/// soland reducer projects this to a `cx.capability.grant` event.
pub async fn attach_personal_agent_grant(
    id: &str,
    action: &str,
    scope: Option<&str>,
) -> Result<AgentGrantEntry, HttpError> {
    let url = format!("/api/v1/agents/{}/grants", urlencoding::encode(id));
    let body = serde_json::json!({
        "action": action,
        "scope": scope,
    });
    api_client(&url, "POST", Some(body.to_string())).await
}

/// `POST /api/v1/agents/{id}/sidecar-thread/ensure` —
/// `cx.agent.sidecar_thread.ensure`.
pub async fn ensure_sidecar_thread(id: &str) -> Result<serde_json::Value, HttpError> {
    let url = format!(
        "/api/v1/agents/{}/sidecar-thread/ensure",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

/// coauth `POST /api/v1/agents/{id}/accountability-grant` — step 3 of
/// the provisioning wizard. Issued by coauth (7c9adf7); the request
/// MUST carry a sodmin/soland Bearer token (`cx.agent.manage` scope).
pub async fn issue_accountability_grant(
    agent_id: &str,
    req: &AccountabilityGrantRequest,
) -> Result<AccountabilityGrantResponse, HttpError> {
    // coauth lives on a separate origin; resolved through the same
    // `build_url` helper but using the coauth base prefix.
    let url = format!(
        "/coauth/api/v1/agents/{}/accountability-grant",
        urlencoding::encode(agent_id)
    );
    let body = json_body(req)?;
    api_client(&url, "POST", Some(body)).await
}
