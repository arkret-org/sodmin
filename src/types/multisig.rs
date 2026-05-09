//! DTO shapes for the multi-sig partial-signature aggregation admin
//! surface (Stream H', H'9).
//!
//! When a Space's anchorer cell is configured as `threshold(k of n)` or
//! `mixed`, soland's anchorer worker collects partial signatures from the
//! n DIDs and assembles a fully-signed Anchor only after k partials
//! arrive. This describe surface lets an operator see which Anchors are
//! still waiting on partials, how many have been collected, who hasn't
//! signed yet, and lets the operator submit their own partial signature.
//!
//! Hand-written for now; will become `pub use soland_admin_types::*;`
//! once the admin-types crate lands. See `_todos.md` A0 checklist.

use serde::{Deserialize, Serialize};

/// One row in the multi-sig pending list — an Anchor for which the
/// anchorer worker is collecting partial signatures.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PendingMultisigAnchor {
    pub anchor_id: String,
    pub space_id: String,
    /// Threshold `k` (signatures required).
    pub threshold_k: u32,
    /// Threshold `n` (members in the anchorer set).
    pub threshold_n: u32,
    /// Number of partial signatures collected so far.
    #[serde(default)]
    pub collected_partials: u32,
    /// DIDs that have already submitted partials.
    #[serde(default)]
    pub signers: Vec<String>,
    /// DIDs that have NOT yet signed. Length is always
    /// `threshold_n - collected_partials` when soland populates faithfully.
    #[serde(default)]
    pub missing_signers: Vec<String>,
    /// State_root the partials are signing over. Surfaced in the UI so
    /// operators can sanity-check before signing.
    #[serde(default)]
    pub state_root: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    /// Whether the *current* admin DID (resolved by the auth bearer
    /// token) is one of the missing signers. soland populates this so
    /// the UI doesn't have to re-resolve the DID client-side.
    #[serde(default)]
    pub admin_can_sign: bool,
}

impl PendingMultisigAnchor {
    /// Convenience: how many additional partials are still needed before
    /// the threshold is met. `0` when the threshold is satisfied (and
    /// soland is just lagging on assembling the full signature).
    pub fn remaining(&self) -> u32 {
        self.threshold_k.saturating_sub(self.collected_partials)
    }

    /// Whether this row has reached the `k` threshold and is awaiting
    /// final assembly (vs still actively collecting). Useful for the UI
    /// badge variant.
    pub fn is_threshold_met(&self) -> bool {
        self.collected_partials >= self.threshold_k
    }

    /// Short `k of n` label for display.
    pub fn threshold_label(&self) -> String {
        format!("{} of {}", self.threshold_k, self.threshold_n)
    }
}

/// Body POSTed to `/api/admin/v1/spaces/{id}/multisig/{anchor_id}/partial`.
/// soland resolves the admin's DID from the bearer token, signs the
/// anchor's `state_root` with the admin's bound signing key, and folds
/// the resulting partial into the pending Anchor's signature set.
///
/// The body is intentionally minimal — a single optional `note` so the
/// operator can leave a comment in the audit log. The signer's DID and
/// signature material is computed server-side from the admin scope.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitPartialSignatureRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Response shape for the partial-signature submit endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitPartialSignatureResponse {
    pub anchor_id: String,
    pub collected_partials: u32,
    pub threshold_k: u32,
    /// Whether the threshold has now been met (soland will assemble the
    /// final Anchor signature on the next worker tick).
    #[serde(default)]
    pub threshold_met: bool,
    #[serde(default)]
    pub signer_did: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_saturates_at_zero_past_threshold() {
        let p = PendingMultisigAnchor {
            threshold_k: 3,
            collected_partials: 5,
            ..Default::default()
        };
        // saturating_sub means "extra" partials don't underflow into a
        // huge u32 — we just clamp to 0.
        assert_eq!(p.remaining(), 0);
        assert!(p.is_threshold_met());
    }

    #[test]
    fn remaining_counts_missing_when_below_threshold() {
        let p = PendingMultisigAnchor {
            threshold_k: 3,
            threshold_n: 5,
            collected_partials: 1,
            ..Default::default()
        };
        assert_eq!(p.remaining(), 2);
        assert!(!p.is_threshold_met());
    }

    #[test]
    fn threshold_label_renders_k_of_n() {
        let p = PendingMultisigAnchor {
            threshold_k: 2,
            threshold_n: 3,
            ..Default::default()
        };
        assert_eq!(p.threshold_label(), "2 of 3");
    }

    #[test]
    fn submit_partial_request_omits_empty_note() {
        let req = SubmitPartialSignatureRequest { note: None };
        let s = serde_json::to_string(&req).unwrap();
        // None notes are skipped on the wire so the backend can default
        // them; the wire body is just `{}` for the bare submit case.
        assert_eq!(s, "{}");

        let req = SubmitPartialSignatureRequest {
            note: Some("rotated key 2026-05-09".into()),
        };
        let s = serde_json::to_string(&req).unwrap();
        assert!(s.contains("\"note\":\"rotated key 2026-05-09\""));
    }

    #[test]
    fn threshold_exact_match_is_met() {
        // Boundary case: collected == k means the threshold is satisfied
        // and soland will assemble next tick.
        let p = PendingMultisigAnchor {
            threshold_k: 2,
            collected_partials: 2,
            ..Default::default()
        };
        assert!(p.is_threshold_met());
        assert_eq!(p.remaining(), 0);
    }
}
