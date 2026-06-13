use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::net::error::HttpError;

fn json_body<T: serde::Serialize>(value: &T) -> Result<String, HttpError> {
    serde_json::to_string(value).map_err(|e| HttpError::message(format!("serialize: {e}")))
}

pub async fn list_agents(page: u64, per_page: u64) -> Result<ListResponse<Agent>, HttpError> {
    let url = build_url(
        "/_soland/admin/agents",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn get_agent(id: &str) -> Result<Agent, HttpError> {
    let url = format!("/_soland/admin/agents/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

// ── CKP-0008 personal-agent surface (soland P2 aa76b91) ──
//
// The 11 new soland endpoints. The admin UI supplies explicit
// controller DID / key proof / grant action form data before exercising
// these calls so privileged agent operations are not fired by accident.

/// `GET /_cokret/self/agents` — list controller-self native personal agents.
pub async fn list_personal_agents(
    page: u64,
    per_page: u64,
) -> Result<ListResponse<Agent>, HttpError> {
    let url = build_url(
        "/_cokret/self/agents",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

/// `GET /_cokret/self/agents/{id}` — controller-self detail.
pub async fn get_personal_agent(id: &str) -> Result<Agent, HttpError> {
    let url = format!("/_cokret/self/agents/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

/// `POST /_cokret/self/agents` — `ck.self.agent.command.provision`. Request and
/// response are the SDK-authoritative `AgentProvisionRequestBody` /
/// `AgentProvisionOutcome` (the controller is the authenticated
/// principal; the outcome carries the pairing handshake). Step-3 of the
/// wizard completes via the coauth `accountability_grant` call below.
pub async fn provision_personal_agent(
    req: &AgentProvisionRequestBody,
) -> Result<AgentProvisionOutcome, HttpError> {
    let body = json_body(req)?;
    api_client("/_cokret/self/agents", "POST", Some(body)).await
}

/// `POST /_cokret/self/agents/{id}/pause` — `ck.self.agent.command.pause`.
pub async fn pause_personal_agent(id: &str) -> Result<(), HttpError> {
    let url = format!("/_cokret/self/agents/{}/pause", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

/// `POST /_cokret/self/agents/{id}/resume` — `ck.self.agent.command.resume`.
pub async fn resume_personal_agent(id: &str) -> Result<(), HttpError> {
    let url = format!("/_cokret/self/agents/{}/resume", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

/// `POST /_cokret/self/agents/{id}/deactivate` — `ck.self.agent.command.deactivate`.
/// Destructive: callers MUST gate this through `ConfirmDialog` with
/// typed-keyword confirmation (`DEACTIVATE`).
pub async fn deactivate_personal_agent(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_cokret/self/agents/{}/deactivate",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

/// `POST /_cokret/self/agents/{id}/rotate-key` — `ck.self.agent.command.rotate_key`.
/// soland's outcome is its own rotate-key shape (`ok` /
/// `authorized_verification_method` / ...), not the provision outcome;
/// current callers only branch on success, so the payload stays untyped
/// until the rotate flow grows a real UI.
pub async fn rotate_personal_agent_key(id: &str) -> Result<serde_json::Value, HttpError> {
    let url = format!(
        "/_cokret/self/agents/{}/rotate-key",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

/// `POST /_cokret/self/agents/{id}/grants` — `ck.self.agent.grant.command.attach`.
/// `action` is one of the 14 personal-agent capability actions; the
/// soland reducer projects this to a `ck.capability.grant` event.
pub async fn attach_personal_agent_grant(
    id: &str,
    action: &str,
    scope: Option<&str>,
) -> Result<AgentGrantEntry, HttpError> {
    let url = format!("/_cokret/self/agents/{}/grants", urlencoding::encode(id));
    let body = serde_json::json!({
        "action": action,
        "scope": scope,
    });
    api_client(&url, "POST", Some(body.to_string())).await
}

/// Ensure the personal-agent sidecar thread via the spec protocol-plane
/// endpoint `POST /_cokret/self/agent-sidecar-threads:ensure`
/// (`ck.self.agent.sidecar_thread.command.ensure`). The idempotency key
/// is server-derived from `(realm_id, controller_principal_id)`; the
/// agent principal is carried in the body, not the path.
pub async fn ensure_sidecar_thread(id: &str) -> Result<serde_json::Value, HttpError> {
    let body = serde_json::json!({ "agent_principal_id": id });
    api_client(
        "/_cokret/self/agent-sidecar-threads:ensure",
        "POST",
        Some(body.to_string()),
    )
    .await
}

/// coauth `POST /_coauth/self/agents/{id}/accountability-grant` — step 3 of
/// the provisioning wizard. Issued by coauth (7c9adf7) on the `/_coauth/`
/// product plane (reverse-proxied to coauth by the sodmin nginx).
///
/// NOTE(SOD-06-010, 待决策): coauth gates this endpoint to soland/sodmin
/// static S2S bearer tokens and rejects browser sessions with 401. The
/// sodmin SPA is cookie-only with no `Authorization: Bearer` path, so the
/// authentication layer for this call still needs a server-side proxy
/// decision upstream. The URL and wire shape below are aligned now; the
/// auth model remains a pending cross-repo decision.
pub async fn issue_accountability_grant(
    agent_id: &str,
    req: &AccountabilityGrantRequest,
) -> Result<AccountabilityGrantOutcome, HttpError> {
    let url = format!(
        "/_coauth/self/agents/{}/accountability-grant",
        urlencoding::encode(agent_id)
    );
    let body = json_body(req)?;
    api_client(&url, "POST", Some(body)).await
}
