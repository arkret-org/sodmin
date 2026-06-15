//! DTO shapes for the Realm `media_service` admin surface.
//!
//! Mirrors the soland read-only endpoint
//! `GET /_soland/admin/realms/{id}/media-service`, which projects the
//! effective `ck.component.realm.media_service.v1` cell. This surface is
//! read-only in sodmin: media_service writes strand through events /
//! yougen, not the operations console.

use serde::{Deserialize, Serialize};

/// One focus in a Realm's `media_service.foci[]` set. Each focus binds a
/// single SFU backend the Realm advertises to clients
/// (`media-service-binding.md` §2).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MediaServiceFocus {
    #[serde(default)]
    pub focus_id: Option<String>,
    /// SFU backend kind, e.g. `livekit` / `mediasoup` / `janus` /
    /// `cokret-native` / `moq-relay`.
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub connect_url: Option<String>,
    /// Signing key id the focus issues participant tokens under.
    #[serde(default)]
    pub issuer_kid: Option<String>,
    #[serde(default)]
    pub audience: Option<String>,
    /// Regions the focus serves, when the Realm declares them.
    #[serde(default)]
    pub regions: Vec<String>,
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
