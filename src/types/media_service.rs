//! Realm `media_service` admin surface — contract-authoritative types.
//!
//! The shared `soland_contracts::admin` types back
//! `GET /_soland/admin/realms/{id}/media-service`, which projects the
//! effective `ak.component.realm.media_service.v1` cell. This surface is
//! read-only in sodmin: media_service writes strand through events /
//! inkson, not the operations console.

pub use soland_contracts::admin::AdminRealmMediaService;
