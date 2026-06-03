//! CXP-0007 Circle types — wire shapes for the soland
//! `/_cokret/self/circles/*` admin surface.
//!
//! A Circle is an encrypted sub-boundary inside a Realm (the security
//! boundary). It carries its own MLS group, member list, and lifecycle
//! state (`active` / `archived` / `tombstoned`). Membership must always
//! be a strict subset of the parent Realm — the soland reducer enforces
//! this via `circle_member_must_be_realm_member`; the admin UI shows
//! the constraint up front so operators get an early hint before the
//! server-side rejection comes back.
//!
//! These structs deserialise the exact shapes emitted by
//! `crate::routing::circles` in soland (see the matching
//! `CircleResponse` / `ListCirclesResponse` / `CreateCircleRequest`
//! definitions there). Lifecycle state strings track the spec's
//! `cx.circle.state` registry — see
//! `cokret-spec/spec/v1/artifacts/registry/circle-state-registry.json`.
//!
//! P3A.7 naming rule: typed entity references inside this crate use
//! the `_id` suffix (`realm_id`, `circle_id`, `scope_circle_id`,
//! `actor_id`) matching the SDK §1.6 convention. The legacy `_ref`
//! suffix is forbidden for `allowed_circle_ids` (a plural list of typed
//! refs inside a GrantConstraint) — the only place the SDK still uses
//! that token.

use serde::{Deserialize, Serialize};

/// One Circle row, mirror of soland's `CircleResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Circle {
    /// `ck:circle:<ulid>` identifier.
    #[serde(default)]
    pub circle_id: String,
    /// Parent Realm (`ck:realm:<ulid>`). Immutable for the life of the
    /// Circle.
    #[serde(default)]
    pub realm_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub summary: Option<String>,
    /// `members` / `realm` / `public`.
    #[serde(default)]
    pub directory_visibility: String,
    /// `invite` / `knock` / `closed`.
    #[serde(default)]
    pub join_rule: String,
    /// `joined` / `invited` / `world_readable` / `shared`.
    #[serde(default)]
    pub history_visibility: String,
    #[serde(default)]
    pub metadata_encryption_floor: Option<String>,
    /// Encryption profile (default `mls_rfc9420`).
    #[serde(default)]
    pub encryption_profile: String,
    /// Reference into the MLS group store; rotated by the
    /// `scope-rotate` admin action.
    #[serde(default)]
    pub mls_group_ref: Option<String>,
    /// `active` / `archived` / `tombstoned`.
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub created_by: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_by: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl Circle {
    /// Returns `true` if the Circle is still mutable (active state).
    /// `archived` and `tombstoned` Circles reject all admin actions
    /// other than `scope-rotate` (archived only).
    pub fn is_active(&self) -> bool {
        self.state == "active"
    }

    pub fn is_tombstoned(&self) -> bool {
        self.state == "tombstoned"
    }
}

/// `GET /_cokret/self/circles?realm_id=...` response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListCirclesResponse {
    #[serde(default)]
    pub realm_id: String,
    #[serde(default)]
    pub circles: Vec<Circle>,
}

/// `POST /_cokret/self/circles` request body. Defaults mirror the soland
/// handler — operators can omit visibility / join_rule and let the
/// server apply the spec's defaults.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateCircleRequest {
    pub realm_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_encryption_floor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_profile: Option<String>,
}

/// `POST /_cokret/self/circles/{id}/members` request body.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CircleMemberRequest {
    pub actor_id: String,
    /// `invited` / `active` / `removed` / `banned` / `left`. Defaults
    /// to `active` server-side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// `POST /_cokret/self/circles/{id}/members` response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CircleMembershipResponse {
    #[serde(default)]
    pub circle_id: String,
    #[serde(default)]
    pub actor_id: String,
    #[serde(default)]
    pub state: String,
}

/// `POST /_cokret/self/circles/{id}/scope-rotate` response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CircleScopeRotateResponse {
    #[serde(default)]
    pub circle_id: String,
    #[serde(default)]
    pub mls_group_ref: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_state_helpers() {
        let mut c = Circle {
            state: "active".into(),
            ..Default::default()
        };
        assert!(c.is_active() && !c.is_tombstoned());
        c.state = "archived".into();
        assert!(!c.is_active() && !c.is_tombstoned());
        c.state = "tombstoned".into();
        assert!(!c.is_active() && c.is_tombstoned());
    }

    #[test]
    fn list_response_round_trip() {
        let raw = serde_json::json!({
            "realm_id": "ck:realm:r1",
            "circles": [{
                "circle_id": "ck:circle:c1",
                "realm_id": "ck:realm:r1",
                "title": "Trust & Safety",
                "directory_visibility": "members",
                "join_rule": "invite",
                "history_visibility": "joined",
                "encryption_profile": "mls_rfc9420",
                "state": "active",
                "members": ["did:ck:alice"],
                "created_by": "did:ck:admin",
                "created_at": "2026-05-01T00:00:00Z"
            }]
        });
        let resp: ListCirclesResponse = serde_json::from_value(raw).expect("deserialise");
        assert_eq!(resp.circles.len(), 1);
        assert!(resp.circles[0].is_active());
    }

    #[test]
    fn create_request_omits_none() {
        let req = CreateCircleRequest {
            realm_id: "ck:realm:r1".into(),
            title: "T&S".into(),
            ..Default::default()
        };
        let wire = serde_json::to_string(&req).unwrap();
        assert!(wire.contains("realm_id"));
        assert!(!wire.contains("summary"));
        assert!(!wire.contains("directory_visibility"));
    }
}
