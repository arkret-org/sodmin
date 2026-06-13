//! DTO shapes for the agent admin surface.

use serde::{Deserialize, Serialize};

// ── Agent types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Agent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub owner_id: String,
    #[serde(default)]
    pub agent_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_active_at: Option<String>,
    /// CKP-0008 — reducer-stamped actor kind. Native personal agents
    /// use `agent`; Applet-managed ghost actors use `integration` or
    /// `agent` plus provenance/accountability metadata.
    /// Populated by soland's `ck.self.agent.query.list` / `ck.self.agent.resource.get`.
    #[serde(default)]
    pub actor_kind: Option<String>,
    /// CKP-0008 — controller DID. Native personal agents are 1:1 bound
    /// to a controller DID; Applet-managed actors point at their owning
    /// applet or integration provenance instead.
    #[serde(default)]
    pub controller_did: Option<String>,
    /// CKP-0008 — current `accountability_grant` id (coauth-issued).
    /// `None` when no grant has been issued / the existing one was
    /// revoked.
    #[serde(default)]
    pub accountability_grant_id: Option<String>,
    /// CKP-0008 — ISO-8601 timestamp of when the
    /// `accountability_grant` was last refreshed. Drives the
    /// "accountability grant freshness" indicator on the detail page.
    #[serde(default)]
    pub accountability_grant_refreshed_at: Option<String>,
    /// CKP-0008 — current pairing status (e.g. `paired`, `pending`,
    /// `expired`). Surfaced verbatim on the detail page.
    #[serde(default)]
    pub pairing_status: Option<String>,
    /// CKP-0008 — list of authorized agent key DIDs.
    #[serde(default)]
    pub agent_keys: Vec<String>,
}

/// CKP-0008 — capability grant detail for the personal-agent detail
/// view's grant editor. Mirrors the soland `agent.grant.attach` /
/// `agent.grant.detach` payload.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentGrantEntry {
    #[serde(default)]
    pub grant_id: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub issued_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

// `POST /_cokret/self/agents` (`ck.self.agent.command.provision`) wire shapes —
// SDK-authoritative per spec `agent-operations.schema.json#/$defs/
// agent_provision_request_body` / `agent_provision_outcome`. The
// controller is always the authenticated principal (no controller_did in
// the body); the outcome carries the pairing handshake
// (`pairing_request_id` / `pairing_code` / `expires_at`).
pub use cokret_core::model::{AgentProvisionOutcome, AgentProvisionRequestBody};

/// CKP-0008 — coauth `accountability_grant` request body for the
/// wizard's controller-approval step. Mirrors coauth's
/// `AccountabilityGrantRequestBody` (`controller_did` / required
/// `capabilities` / optional `reason`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountabilityGrantRequest {
    #[serde(default)]
    pub controller_did: String,
    /// Capability actions covered by the grant. Each entry must be a
    /// registered `ck.agent.*` action from `capability-action-registry.json`.
    /// Required by coauth (no serde default upstream).
    pub capabilities: Vec<String>,
    /// Optional human-readable reason recorded with the grant for the
    /// audit trail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountabilityGrantOutcome {
    #[serde(default)]
    pub accountability_grant_id: String,
    #[serde(default)]
    pub issued_at: Option<String>,
}
