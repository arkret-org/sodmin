//! DTO shapes for the delivery-binding admin surface.

use arkret_core::ErrorCode;
use serde::{Deserialize, Serialize};

// ── Delivery binding policy (T6.2 §3) ──

/// Effective `ak.component.realm.delivery_binding_policy.v1` for a Realm.
/// `allowed_recipient_services` and `binding_source_policy` are
/// operator-mutable; `policy_frontier` is written by the soland
/// reducer and is therefore read-only on the admin surface.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealmDeliveryBindingPolicy {
    /// Realm identifier (security boundary).
    #[serde(default)]
    pub realm_id: String,
    #[serde(default)]
    pub allowed_recipient_services: Vec<String>,
    #[serde(default)]
    pub binding_source_policy: Option<String>,
    #[serde(default)]
    pub policy_frontier: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// One row in the per-Realm "is each member routable?" check table.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemberRoutabilityRow {
    #[serde(default)]
    pub actor_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub recipient_service_did: Option<String>,
    #[serde(default)]
    pub in_allowed_list: bool,
    #[serde(default)]
    pub delivery_status: Option<String>,
}

// ── Round 4 — Delivery binding handover (error codes
//     delivery_binding_stale / delivery_binding_handed_over /
//     historical_only) ──────────────────────────────────────────────

/// Round 4 — one row in the delivery-binding handover panel. Surfaces
/// the new error-code triple plus the redirect target + frontier the
/// handover advertises.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DeliveryBindingHandoverRow {
    #[serde(default)]
    pub realm_id: String,
    #[serde(default)]
    pub actor_id: String,
    /// The previous (now-stale) recipient service DID, if known.
    #[serde(default)]
    pub previous_recipient_service_did: Option<String>,
    /// `new_recipient_service_did` — the redirect target the recipient
    /// service advertises in the `delivery_binding_stale` /
    /// `delivery_binding_handed_over` 409 body.
    #[serde(default)]
    pub new_recipient_service_did: Option<String>,
    /// `handover_frontier` — the frontier (vector of `ak:event:*` refs)
    /// at/after which the new recipient takes effect.
    #[serde(default)]
    pub handover_frontier: Vec<String>,
    #[serde(default)]
    pub reason_code: Option<String>,
    #[serde(default)]
    pub observed_at: Option<String>,
}

impl DeliveryBindingHandoverRow {
    pub fn classified_reason(&self) -> Option<ErrorCode> {
        self.reason_code
            .as_deref()
            .and_then(ErrorCode::from_wire)
            .filter(|reason| {
                matches!(
                    *reason,
                    ErrorCode::DeliveryBindingStale
                        | ErrorCode::DeliveryBindingHandedOver
                        | ErrorCode::HistoricalOnly
                )
            })
    }
}
