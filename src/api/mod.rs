pub mod actors;
pub mod agents;
pub mod applets;
pub mod applets_agents_directory;
pub mod audit;
pub mod auth;
pub mod authz;
pub mod capabilities;
pub mod circles;
pub mod client;
pub mod coauth;
pub mod coauth_devices;
pub mod contracts;
pub mod covered_seals;
pub mod delivery_binding;
pub mod devices;
/// R3.2 (UI-SOD-4) — Directory service client
/// (`ck.find.directory.query.list_handles_for_subject`).
pub mod directory;
pub mod federation;
pub mod federation_status;
pub mod handles;
pub mod invite_tokens;
/// B-C key-backup recovery admin (soland P2 series + recovery policy).
pub mod key_backup;
pub mod media;
pub mod moderation;
pub mod multisig;
pub mod paths;
pub mod policy;
pub mod realm_links;
pub mod realm_policy;
pub mod realms;
pub mod seal;
pub mod server;
pub mod signing_key;
pub mod spaces;
pub mod starid;
