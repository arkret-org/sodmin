pub mod actors;
pub mod agents;
pub mod anchor_bottom;
pub mod applets;
pub mod audit;
pub mod capabilities;
pub mod circles;
pub mod coauth;
pub mod dashboard;
pub mod deactivation_review;
pub mod delivery_binding;
pub mod devices;
pub mod directory;
pub mod federation;
pub mod handles;
/// R3.2 (UI-SOD-4) — Subject → Handles directory page
/// (`ck.find.directory.list_handles_for_subject`).
pub mod handles_by_subject;
pub mod hardening;
pub mod invite_tokens;
/// B-C key-backup recovery admin (P3-B).
pub mod key_backup;
pub mod login;
pub mod media;
pub mod moderation;
pub mod not_authorized;
pub mod oauth_callback;
pub mod policy;
pub mod realm_destroy;
/// R3.1 (MID-3) — Realm identity audit diagnostic page.
pub mod realm_identity_audit;
pub mod realm_links;
/// R3 (UI-3) — Realm `media_service.foci[]` editor.
pub mod realm_media_service;
pub mod realms;
pub mod server_status;
pub mod spaces;
pub mod starid_resolver;
