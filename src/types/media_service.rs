//! DTO shapes for the Realm `media_service` admin surface.
//!
//! Mirrors the soland read-only endpoint
//! `GET /_soland/admin/realms/{id}/media-service`, which projects the
//! effective `ak.component.realm.media_service.v1` cell. This surface is
//! read-only in sodmin: media_service writes strand through events /
//! inkson, not the operations console.

use serde::{Deserialize, Serialize};

/// One focus in a Realm's `media_service.foci[]` set. Each focus binds a
/// single SFU backend the Realm advertises to clients
/// (`media-service-binding.md` §2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaServiceFocus {
    pub focus_id: String,
    /// SFU backend kind, e.g. `livekit` / `mediasoup` / `janus` /
    /// `arkret-native` / `moq-relay`.
    #[serde(rename = "type")]
    pub focus_type: String,
    pub region: Option<String>,
    pub token_endpoint: String,
    pub connect_url: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub health_endpoint: Option<String>,
    pub cascade_group: Option<String>,
}

/// Effective `ak.component.realm.media_service.v1` cell for a Realm,
/// surfaced read-only for sodmin. `service_id`,
/// `e2ee_key_sources_allowed` and `foci` are all reducer-projected;
/// nothing here is operator-mutable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmMediaService {
    /// Realm identifier (security boundary).
    pub realm_id: String,
    /// Media service DID the foci are sealed to (absent until the Realm
    /// commits a media_service epoch).
    pub service_id: Option<String>,
    #[serde(default)]
    pub e2ee_key_sources_allowed: Vec<String>,
    #[serde(default)]
    pub foci: Vec<MediaServiceFocus>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_requires_normative_binding_fields() {
        for missing in ["focus_id", "type", "token_endpoint", "connect_url"] {
            let mut value = serde_json::json!({
                "focus_id": "fra-1",
                "type": "livekit",
                "token_endpoint": "https://media.example/_arkret/self/rtc/token",
                "connect_url": "wss://media.example"
            });
            value
                .as_object_mut()
                .expect("focus fixture is an object")
                .remove(missing);
            assert!(
                serde_json::from_value::<MediaServiceFocus>(value).is_err(),
                "missing {missing} must fail closed"
            );
        }
    }

    #[test]
    fn optional_focus_metadata_may_be_absent() {
        let focus: MediaServiceFocus = serde_json::from_value(serde_json::json!({
            "focus_id": "fra-1",
            "type": "livekit",
            "token_endpoint": "https://media.example/_arkret/self/rtc/token",
            "connect_url": "wss://media.example"
        }))
        .expect("optional focus metadata may be absent");
        assert!(focus.capabilities.is_empty());
        assert!(focus.region.is_none());
    }
}
