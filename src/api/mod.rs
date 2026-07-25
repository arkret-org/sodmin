pub mod actors;
pub mod audit;
pub mod auth;
pub mod capabilities;
pub mod client;
pub mod coauth;
pub mod coauth_devices;
pub mod covered_seals;
pub mod delivery_binding;
pub mod devices;
/// R3.2 (UI-SOD-4) — Directory service client
/// (`ak.find.directory.query.list_handles_for_subject`).
pub mod directory;
pub mod federation;
pub mod handles;
pub mod invite_tokens;
/// B-C key-backup recovery admin (soland P2 series + recovery policy).
pub mod key_backup;
pub mod media;
pub mod media_service;
pub mod multisig;
pub mod policy;
pub mod realm_links;
/// Realm organization relationships and principal-control projections.
pub mod realm_organization;
pub mod realms;
pub mod seal;
pub mod server;
pub mod spaces;
pub mod starid;
