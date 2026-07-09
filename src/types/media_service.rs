//! DTO shapes for the Realm `media_service` admin surface.
//!
//! Mirrors the soland read-only endpoint
//! `GET /_soland/admin/realms/{id}/media-service`, which projects the
//! effective `ck.component.realm.media_service.v1` cell. This surface is
//! read-only in sodmin: media_service writes strand through events /
//! inkson, not the operations console.

use serde::{Deserialize, Serialize};

/// One focus in a Realm's `media_service.foci[]` set. Each focus binds a
/// single SFU backend the Realm advertises to clients
/// (`media-service-binding.md` §2).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MediaServiceFocus {
    #[serde(default)]
    pub focus_id: Option<String>,
    /// SFU backend kind, e.g. `livekit` / `mediasoup` / `janus` /
    /// `arkret-native` / `moq-relay`.
    #[serde(default, rename = "type")]
    pub focus_type: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub token_endpoint: Option<String>,
    #[serde(default)]
    pub connect_url: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub health_endpoint: Option<String>,
    #[serde(default)]
    pub cascade_group: Option<String>,
}

/// Effective `ck.component.realm.media_service.v1` cell for a Realm,
/// surfaced read-only for sodmin. `service_id`,
/// `e2ee_key_sources_allowed` and `foci` are all reducer-projected;
/// nothing here is operator-mutable.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealmMediaService {
    /// Realm identifier (security boundary).
    #[serde(default)]
    pub realm_id: String,
    /// Media service DID the foci are sealed to (absent until the Realm
    /// commits a media_service epoch).
    #[serde(default)]
    pub service_id: Option<String>,
    #[serde(default)]
    pub e2ee_key_sources_allowed: Vec<String>,
    #[serde(default)]
    pub foci: Vec<MediaServiceFocus>,
}
