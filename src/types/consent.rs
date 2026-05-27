//! DTO shapes for the consent admin surface.
//!
//! These mirror the join-projection of `cx:cell:cx.component.consent.v1:<holder_did>`
//! consent or-set values that soland exposes via the admin describe endpoint.
//!
//! IMPORTANT: this is *admin-visible aggregated metadata*, not the raw
//! peer-relations a holder has granted. The soland-side endpoint is
//! responsible for redaction (it only joins the public or-set value, not
//! holder-private peer addressing).

use serde::{Deserialize, Serialize};

/// Status of a single consent grant entry. Matches soland's reducer enum
/// for `cx.component.consent.v1` cells.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStatus {
    Active,
    Revoked,
    Expired,
    Pending,
}

impl ConsentStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ConsentStatus::Active => "Active",
            ConsentStatus::Revoked => "Revoked",
            ConsentStatus::Expired => "Expired",
            ConsentStatus::Pending => "Pending",
        }
    }

    /// Parse from the wire enum string. `None` if the string is unknown.
    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "active" => Some(ConsentStatus::Active),
            "revoked" => Some(ConsentStatus::Revoked),
            "expired" => Some(ConsentStatus::Expired),
            "pending" => Some(ConsentStatus::Pending),
            _ => None,
        }
    }
}

/// One row in the consent admin panel — a join over the consent cell's
/// or-set value for a single (holder, peer, scope) triple.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsentGrant {
    pub holder_did: String,
    pub peer_did: String,
    pub scope: String,
    /// Wire-format `ConsentStatus` (snake_case).
    pub status: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    /// Stable id for admin-side mutations (resolve route). soland is
    /// expected to populate this on rows that are eligible for the
    /// `consent_admin::resolve` override (today: only `Pending` rows).
    #[serde(default)]
    pub consent_id: Option<String>,
}

/// Admin override decision for a `Pending` consent row.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsentResolveDecision {
    Approve,
    Reject,
}

/// Body POSTed to `/api/admin/v1/consent/{consent_id}/resolve`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentResolveRequest {
    pub decision: ConsentResolveDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl ConsentGrant {
    /// Typed status, falling back to `Pending` for unknown values so the
    /// admin sees *something* in the badge column rather than an empty
    /// cell.
    pub fn status_typed(&self) -> ConsentStatus {
        ConsentStatus::from_wire(&self.status).unwrap_or(ConsentStatus::Pending)
    }
}

/// Apply the holder filter to a slice of grants. Empty filter passes
/// everything through; non-empty filter does a case-insensitive `contains`
/// match against the holder DID. Pure function so we can unit-test it
/// without spinning up the page.
pub fn filter_by_holder(grants: &[ConsentGrant], holder_filter: &str) -> Vec<ConsentGrant> {
    let needle = holder_filter.trim().to_lowercase();
    if needle.is_empty() {
        return grants.to_vec();
    }
    grants
        .iter()
        .filter(|g| g.holder_did.to_lowercase().contains(&needle))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consent_status_from_wire_round_trips() {
        for (wire, expected_label) in [
            ("active", "Active"),
            ("revoked", "Revoked"),
            ("expired", "Expired"),
            ("pending", "Pending"),
        ] {
            let s = ConsentStatus::from_wire(wire).expect("wire variant");
            assert_eq!(s.label(), expected_label);
        }
        assert!(ConsentStatus::from_wire("nope").is_none());
        // case-sensitive on the wire — we never see uppercase from soland.
        assert!(ConsentStatus::from_wire("ACTIVE").is_none());
    }

    #[test]
    fn consent_grant_status_typed_falls_back_to_pending() {
        let g = ConsentGrant {
            status: "garbage".into(),
            ..Default::default()
        };
        assert_eq!(g.status_typed(), ConsentStatus::Pending);

        let g = ConsentGrant {
            status: "revoked".into(),
            ..Default::default()
        };
        assert_eq!(g.status_typed(), ConsentStatus::Revoked);
    }

    #[test]
    fn filter_by_holder_empty_returns_everything() {
        let grants = vec![
            ConsentGrant {
                holder_did: "did:cx:alice".into(),
                ..Default::default()
            },
            ConsentGrant {
                holder_did: "did:cx:bob".into(),
                ..Default::default()
            },
        ];
        assert_eq!(filter_by_holder(&grants, "").len(), 2);
        assert_eq!(filter_by_holder(&grants, "   ").len(), 2);
    }

    #[test]
    fn resolve_decision_serializes_snake_case() {
        // The wire shape MUST be `{decision: "approve" | "reject"}`,
        // matching the invite-quarantine resolve route (which soland is
        // expected to copy). Drift here would silently break the admin
        // override.
        let approve = ConsentResolveRequest {
            decision: ConsentResolveDecision::Approve,
            note: Some("looks legit".into()),
        };
        let s = serde_json::to_string(&approve).unwrap();
        assert!(s.contains("\"decision\":\"approve\""));
        assert!(s.contains("\"note\":\"looks legit\""));

        let reject_no_note = ConsentResolveRequest {
            decision: ConsentResolveDecision::Reject,
            note: None,
        };
        let s = serde_json::to_string(&reject_no_note).unwrap();
        assert!(s.contains("\"decision\":\"reject\""));
        // None note is skipped on the wire so the backend can default it.
        assert!(!s.contains("\"note\""));
    }

    #[test]
    fn filter_by_holder_matches_case_insensitive_substring() {
        let grants = vec![
            ConsentGrant {
                holder_did: "did:cx:Alice".into(),
                ..Default::default()
            },
            ConsentGrant {
                holder_did: "did:cx:bob".into(),
                ..Default::default()
            },
            ConsentGrant {
                holder_did: "did:web:example.com:alice2".into(),
                ..Default::default()
            },
        ];
        let out = filter_by_holder(&grants, "alice");
        assert_eq!(out.len(), 2);
        assert!(
            out.iter()
                .all(|g| g.holder_did.to_lowercase().contains("alice"))
        );

        let out = filter_by_holder(&grants, "BOB");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].holder_did, "did:cx:bob");

        let out = filter_by_holder(&grants, "carol");
        assert!(out.is_empty());
    }
}
