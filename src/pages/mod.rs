pub mod actors;
pub mod audit;
pub mod capabilities;
pub mod coauth;
pub mod dashboard;
pub mod deactivation_review;
pub mod devices;
pub mod federation;
pub mod handles;
/// R3.2 (UI-SOD-4) — Subject → Handles directory page
/// (`ck.find.directory.query.list_handles_for_subject`).
pub mod handles_by_subject;
pub mod hardening;
pub mod invite_tokens;
/// B-C key-backup recovery admin (P3-B).
pub mod key_backup;
pub mod login;
pub mod media;
pub mod not_authorized;
pub mod oauth_callback;
pub mod policy;
pub mod realms;
pub mod seal_bottom;
pub mod server_status;
pub mod spaces;
pub mod starid_resolver;
