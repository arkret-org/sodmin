pub mod actors;
pub mod audit;
pub mod capabilities;
pub mod coauth;
pub mod dashboard;
pub mod devices;
pub mod federation;
pub mod handles;
/// Subject → Handles directory page
/// (`ak.find.directory.read.list_handles_for_subject.v1`).
pub mod handles_by_subject;
pub mod hardening;
pub mod invite_tokens;
/// Key-backup recovery admin (backup blobs + recovery policy).
pub mod key_backup;
pub mod login;
pub mod media;
pub mod not_authorized;
pub mod oauth_callback;
pub mod policy;
pub mod realms;
pub mod seal_bottom;
pub mod server_status;
pub mod service_routes;
pub mod spaces;
