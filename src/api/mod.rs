pub mod actors;
pub mod anchor_admin;
pub mod api_client {
    pub use crate::api::client::*;
}
pub mod components_admin;
pub mod consent_admin;
pub mod covered_frontier_admin;
// TODO(A0): populated by codegen once coauth-admin-types / soland-admin-types land.
// See _todos.md A0 checklist.
pub mod generated;
pub mod agents;
pub mod applets;
pub mod audit;
pub mod auth;
pub mod capabilities;
pub mod client;
pub mod coauth;
pub mod devices;
pub mod federation;
pub mod invite_tokens;
pub mod media;
pub mod policy;
pub mod reports;
pub mod server;
pub mod spaces;
