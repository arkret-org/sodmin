//! CKP-0007 Circle admin envelopes.
//!
//! The wire DTOs come from `cokret_core::model`; sodmin only keeps small UI
//! helpers around the canonical spec types.

pub use cokret_core::model::{
    CircleCreateRequestBody as CreateCircleRequest, CircleDirectoryVisibility, CircleJoinRule,
    CircleList as ListCirclesOutcome, CircleMemberRequestBody as CircleMemberRequest,
    CircleMembership, CircleMembershipOutcome, CircleScopeRotateOutcome, CircleState,
    CircleView as Circle, EncryptionProfile, HistoryVisibility,
};

pub fn circle_is_active(circle: &Circle) -> bool {
    matches!(circle.state, CircleState::Active)
}

pub fn circle_is_tombstoned(circle: &Circle) -> bool {
    matches!(circle.state, CircleState::Tombstoned)
}

pub fn circle_state_label_key(state: &CircleState) -> &'static str {
    match state {
        CircleState::Active => "circle.state_active",
        CircleState::Archived => "circle.state_archived",
        CircleState::Tombstoned => "circle.state_tombstoned",
    }
}

pub fn directory_visibility_wire(value: &CircleDirectoryVisibility) -> &'static str {
    match value {
        CircleDirectoryVisibility::Members => "members",
        CircleDirectoryVisibility::RealmMembers => "realm_members",
    }
}

pub fn join_rule_wire(value: &CircleJoinRule) -> &'static str {
    match value {
        CircleJoinRule::Invite => "invite",
        CircleJoinRule::Request => "request",
        CircleJoinRule::Open => "open",
    }
}

pub fn history_visibility_wire(value: &HistoryVisibility) -> &'static str {
    match value {
        HistoryVisibility::WorldReadable => "world_readable",
        HistoryVisibility::Shared => "shared",
        HistoryVisibility::Invited => "invited",
        HistoryVisibility::Joined => "joined",
        HistoryVisibility::Restricted => "restricted",
    }
}

pub fn encryption_profile_wire(value: &EncryptionProfile) -> &'static str {
    match value {
        EncryptionProfile::None => "none",
        EncryptionProfile::MlsRfc9420 => "mls_rfc9420",
        EncryptionProfile::External => "external",
    }
}

pub fn parse_directory_visibility(value: &str) -> Option<CircleDirectoryVisibility> {
    match value {
        "members" => Some(CircleDirectoryVisibility::Members),
        "realm_members" => Some(CircleDirectoryVisibility::RealmMembers),
        _ => None,
    }
}

pub fn parse_join_rule(value: &str) -> Option<CircleJoinRule> {
    match value {
        "invite" => Some(CircleJoinRule::Invite),
        "request" => Some(CircleJoinRule::Request),
        "open" => Some(CircleJoinRule::Open),
        _ => None,
    }
}

pub fn parse_history_visibility(value: &str) -> Option<HistoryVisibility> {
    match value {
        "world_readable" => Some(HistoryVisibility::WorldReadable),
        "shared" => Some(HistoryVisibility::Shared),
        "invited" => Some(HistoryVisibility::Invited),
        "joined" => Some(HistoryVisibility::Joined),
        "restricted" => Some(HistoryVisibility::Restricted),
        _ => None,
    }
}

pub fn parse_membership(value: &str) -> Option<CircleMembership> {
    match value {
        "join" => Some(CircleMembership::Join),
        "invite" => Some(CircleMembership::Invite),
        "knock" => Some(CircleMembership::Knock),
        "leave" => Some(CircleMembership::Leave),
        "ban" => Some(CircleMembership::Ban),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use cokret_core::RealmId;

    use super::*;

    const REALM_ID: &str = "ck:realm:018f4c28-1234-7abc-8def-123456789abc";

    #[test]
    fn circle_state_helpers() {
        assert_eq!(
            circle_state_label_key(&CircleState::Active),
            "circle.state_active"
        );
        assert_eq!(
            circle_state_label_key(&CircleState::Archived),
            "circle.state_archived"
        );
        assert_eq!(
            circle_state_label_key(&CircleState::Tombstoned),
            "circle.state_tombstoned"
        );
    }

    #[test]
    fn list_outcome_uses_core_circle_shape() {
        let raw = serde_json::json!({
            "realm_id": REALM_ID,
            "circles": [{
                "circle_id": "ck:circle:018f4c28-1234-7abc-8def-123456789abc",
                "realm_id": REALM_ID,
                "title": "Trust & Safety",
                "directory_visibility": "members",
                "join_rule": "invite",
                "history_visibility": "joined",
                "encryption_profile": "mls_rfc9420",
                "mls_group_ref": "ck:mls:group:trust",
                "state": "active",
                "members": ["did:web:alice.example"],
                "created_by": "did:web:admin.example",
                "created_at": "2026-05-01T00:00:00Z"
            }]
        });
        let resp: ListCirclesOutcome = serde_json::from_value(raw).expect("deserialise");
        assert_eq!(resp.circles.len(), 1);
        assert!(circle_is_active(&resp.circles[0]));
    }

    #[test]
    fn create_request_omits_none_and_serializes_core_enums() {
        let req = CreateCircleRequest {
            realm_id: RealmId::new(REALM_ID).unwrap(),
            title: "T&S".into(),
            summary: None,
            directory_visibility: Some(CircleDirectoryVisibility::Members),
            join_rule: Some(CircleJoinRule::Invite),
            history_visibility: Some(HistoryVisibility::Joined),
            content_encryption_floor: None,
            metadata_encryption_floor: None,
            encryption_profile: None,
        };
        let wire = serde_json::to_string(&req).unwrap();
        assert!(wire.contains("realm_id"));
        assert!(wire.contains("directory_visibility"));
        assert!(wire.contains("join_rule"));
        assert!(!wire.contains("summary"));
        assert!(!wire.contains("metadata_encryption_floor"));
    }
}
