//! DTO shapes for the 3PID invite admin surface.

use serde::{Deserialize, Serialize};

// ── Round 4 — 3PID invite admin row ─────────────────────────────────

// SDK-authoritative terminal state for an admin-visible 3PID invite.
// Every terminal value MUST be displayed truthfully; in particular
// `send_failed` is a permanent failure (the OOB code was never
// delivered) and the admin UI must not paper it over as success.
pub use cokret_core::model::ThirdPartyInviteTerminalState;

/// Round 4 — admin-visible 3PID invite row. The plaintext 3PID is
/// intentionally absent — the wire never carries it, and the admin UI
/// never reconstructs it. `evidence` is opaque (commitment digest /
/// lookup ref / pepper id) and stays untrusted on the client.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ThirdPartyInviteRow {
    #[serde(default)]
    pub invite_id: String,
    #[serde(default)]
    pub oob_code_kind: Option<String>,
    #[serde(default)]
    pub verification_service_did: Option<String>,
    /// Slug of the terminal state. Use [`Self::classified_state`] for
    /// the strongly-typed enum.
    #[serde(default)]
    pub terminal_state: Option<String>,
    /// Opaque inspection helper: token-commitment / lookup_table_ref /
    /// pepper_id (whichever applies). Never the plaintext.
    #[serde(default)]
    pub evidence: Option<String>,
    #[serde(default)]
    pub observed_at: Option<String>,
}

impl ThirdPartyInviteRow {
    pub fn classified_state(&self) -> Option<ThirdPartyInviteTerminalState> {
        match self.terminal_state.as_deref() {
            Some("claimed") => Some(ThirdPartyInviteTerminalState::Claimed),
            Some("send_failed") => Some(ThirdPartyInviteTerminalState::SendFailed),
            Some("revoked_by_capability_loss") => {
                Some(ThirdPartyInviteTerminalState::RevokedByCapabilityLoss)
            }
            Some("revoked_by_inviter_left") => {
                Some(ThirdPartyInviteTerminalState::RevokedByInviterLeft)
            }
            Some("invalidated_by_rate_limit") => {
                Some(ThirdPartyInviteTerminalState::InvalidatedByRateLimit)
            }
            _ => None,
        }
    }
}
