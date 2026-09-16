pub mod actors;
pub mod audit;
pub mod auth;
/// Realm authority-commit bundle: current governing Station, Realm commit
/// stream head and the double-signed handoff chain.
pub mod authority;
pub mod capabilities;
pub mod client;
pub mod coauth;
pub mod coauth_devices;
pub mod devices;
/// Directory service client
/// (`ak.find.directory.read.list_handles_for_subject.v1`).
pub mod directory;
pub mod federation;
pub mod handles;
pub mod invite_tokens;
/// Key-backup recovery admin (backup blobs + recovery policy).
pub mod key_backup;
pub mod media;
pub mod media_service;
pub mod policy;
pub mod realm_links;
/// Realm organization relationships and principal-control projections.
pub mod realm_organization;
pub mod realms;
pub mod server;
pub mod service_routes;
pub mod spaces;
