//! DTO shapes for the delivery-binding admin surface.

use serde::{Deserialize, Serialize};

// ── Delivery binding policy (T6.2 §3) ──

/// Effective `ck.component.realm.delivery_binding_policy.v1` for a Realm.
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateDeliveryBindingPolicyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_recipient_services: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_source_policy: Option<String>,
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

/// Round 4 — discriminated reason a delivery-binding handover row
/// surfaces. The first two are wire-breaking failures the operator must
/// act on; `HistoricalOnly` is a 200 diagnostic that documents a
/// cached-replay response and MUST NOT be presented as a fresh action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryBindingHandoverReason {
    /// `delivery_binding_stale` (HTTP 409). Recipient rejected the
    /// envelope because its binding has moved on. Retry against
    /// `new_recipient_service_did` at/after `handover_frontier`.
    DeliveryBindingStale,
    /// `delivery_binding_handed_over` (HTTP 409). Recipient has
    /// permanently handed delivery off; submissions MUST switch to
    /// `new_recipient_service_did`.
    DeliveryBindingHandedOver,
    /// `historical_only` (HTTP 200, diagnostic). Cached replay against a
    /// prior key state. Information only — NOT a fresh action.
    HistoricalOnly,
}

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
    /// `handover_frontier` — the frontier (vector of `ck:event:*` refs)
    /// at/after which the new recipient takes effect.
    #[serde(default)]
    pub handover_frontier: Vec<String>,
    #[serde(default)]
    pub reason_code: Option<String>,
    #[serde(default)]
    pub observed_at: Option<String>,
}

impl DeliveryBindingHandoverRow {
    pub fn classified_reason(&self) -> Option<DeliveryBindingHandoverReason> {
        match self.reason_code.as_deref() {
            Some("delivery_binding_stale") => {
                Some(DeliveryBindingHandoverReason::DeliveryBindingStale)
            }
            Some("delivery_binding_handed_over") => {
                Some(DeliveryBindingHandoverReason::DeliveryBindingHandedOver)
            }
            Some("historical_only") => Some(DeliveryBindingHandoverReason::HistoricalOnly),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateDeliveryBindingPolicyRequest;

    #[test]
    fn delivery_binding_policy_request_omits_none() {
        let req = UpdateDeliveryBindingPolicyRequest {
            allowed_recipient_services: Some(vec!["did:web:floria.example".to_string()]),
            binding_source_policy: None,
        };
        let serialized = serde_json::to_string(&req).expect("serializes");
        assert!(serialized.contains("allowed_recipient_services"));
        assert!(!serialized.contains("binding_source_policy"));
    }
}
