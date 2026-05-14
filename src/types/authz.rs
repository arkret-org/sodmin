//! DTO shapes for the soland authz capability admin surface.
//!
//! These mirror the rows soland's authz admin endpoint emits when listing
//! capability grants.
//!
//! The capability-grant admin surface is intentionally distinct from the
//! sodmin "self-issued" capability page — those grants live in coauth's
//! account capabilities table; soland's authz capabilities are scoped to
//! Move/Anchor cell families and gate per-Space writes.

use serde::{Deserialize, Serialize};

/// Status of a single capability grant. Mirrors soland's authz reducer
/// state machine: `Active` while the grant is valid, `Revoked` after an
/// admin revoke (or self-revoke), `Expired` once the wall-clock TTL
/// passed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthzGrantStatus {
    Active,
    Revoked,
    Expired,
}

impl AuthzGrantStatus {
    pub fn label(&self) -> &'static str {
        match self {
            AuthzGrantStatus::Active => "Active",
            AuthzGrantStatus::Revoked => "Revoked",
            AuthzGrantStatus::Expired => "Expired",
        }
    }

    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "active" => Some(AuthzGrantStatus::Active),
            "revoked" => Some(AuthzGrantStatus::Revoked),
            "expired" => Some(AuthzGrantStatus::Expired),
            _ => None,
        }
    }
}

/// One row in the soland authz capability admin panel.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthzCapabilityGrant {
    /// Stable id assigned by soland on first publish; sodmin uses this
    /// for the revoke route.
    #[serde(default)]
    pub grant_id: String,
    #[serde(default)]
    pub holder_did: String,
    #[serde(default)]
    pub peer_did: String,
    #[serde(default)]
    pub scope: String,
    /// Wire-format `AuthzGrantStatus` (snake_case).
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub granted_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

impl AuthzCapabilityGrant {
    /// Typed status with a safe fallback to `Active` for unknown values
    /// so the badge column is never blank.
    pub fn status_typed(&self) -> AuthzGrantStatus {
        AuthzGrantStatus::from_wire(&self.status).unwrap_or(AuthzGrantStatus::Active)
    }

    /// True iff the grant can be revoked right now — only `Active` rows
    /// expose the revoke button.
    pub fn is_revocable(&self) -> bool {
        matches!(self.status_typed(), AuthzGrantStatus::Active) && !self.grant_id.is_empty()
    }
}

/// Multi-axis filter for the authz admin list. Empty fields are dropped
/// before encoding so the wire form only carries what the operator
/// actually filtered on. Pure data — the API client owns the encoding.
#[derive(Debug, Clone, Default)]
pub struct AuthzGrantFilter {
    pub holder: String,
    pub peer: String,
    pub scope: String,
}

impl AuthzGrantFilter {
    pub fn is_empty(&self) -> bool {
        self.holder.trim().is_empty() && self.peer.trim().is_empty() && self.scope.trim().is_empty()
    }
}

/// Apply the filter client-side to an already-fetched slice. The server
/// is the source of truth for filtering, but for the case where soland
/// returns a coarse page and the operator narrows further, we filter
/// client-side too. Pure function so it can be unit-tested without
/// hitting the network.
pub fn filter_grants(
    grants: &[AuthzCapabilityGrant],
    filter: &AuthzGrantFilter,
) -> Vec<AuthzCapabilityGrant> {
    let holder = filter.holder.trim().to_lowercase();
    let peer = filter.peer.trim().to_lowercase();
    let scope = filter.scope.trim().to_lowercase();
    grants
        .iter()
        .filter(|g| holder.is_empty() || g.holder_did.to_lowercase().contains(&holder))
        .filter(|g| peer.is_empty() || g.peer_did.to_lowercase().contains(&peer))
        .filter(|g| scope.is_empty() || g.scope.to_lowercase().contains(&scope))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grant(holder: &str, peer: &str, scope: &str, status: &str) -> AuthzCapabilityGrant {
        AuthzCapabilityGrant {
            grant_id: "g1".into(),
            holder_did: holder.into(),
            peer_did: peer.into(),
            scope: scope.into(),
            status: status.into(),
            ..Default::default()
        }
    }

    #[test]
    fn status_round_trips_via_wire_strings() {
        for (wire, expected_label) in [
            ("active", "Active"),
            ("revoked", "Revoked"),
            ("expired", "Expired"),
        ] {
            let s = AuthzGrantStatus::from_wire(wire).expect("variant");
            assert_eq!(s.label(), expected_label);
        }
        assert!(AuthzGrantStatus::from_wire("nope").is_none());
        // case-sensitive on the wire
        assert!(AuthzGrantStatus::from_wire("ACTIVE").is_none());
    }

    #[test]
    fn only_active_grants_with_id_are_revocable() {
        // The Revoke button must be hidden for non-Active rows and for
        // rows that lack a server-assigned `grant_id` (we'd have nothing
        // to POST).
        let g = grant("alice", "bob", "scope", "active");
        assert!(g.is_revocable());

        let mut g2 = grant("alice", "bob", "scope", "active");
        g2.grant_id.clear();
        assert!(!g2.is_revocable(), "missing grant_id should hide revoke");

        let g = grant("alice", "bob", "scope", "revoked");
        assert!(!g.is_revocable());

        let g = grant("alice", "bob", "scope", "expired");
        assert!(!g.is_revocable());
    }

    #[test]
    fn filter_grants_intersects_all_three_axes() {
        let grants = vec![
            grant("did:cx:Alice", "did:cx:bob", "cx.cell.write", "active"),
            grant("did:cx:alice", "did:cx:carol", "cx.cell.read", "active"),
            grant("did:cx:dan", "did:cx:bob", "cx.cell.write", "active"),
        ];
        let f = AuthzGrantFilter {
            holder: "alice".into(),
            peer: "bob".into(),
            scope: "write".into(),
        };
        let out = filter_grants(&grants, &f);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].holder_did, "did:cx:Alice");

        // Empty filter passes everything through.
        let f = AuthzGrantFilter::default();
        assert_eq!(filter_grants(&grants, &f).len(), 3);
        assert!(f.is_empty());

        // Non-matching filter returns empty.
        let f = AuthzGrantFilter {
            holder: "zzz".into(),
            ..Default::default()
        };
        assert!(filter_grants(&grants, &f).is_empty());
    }
}
