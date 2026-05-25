//! Admin pages for CXP-0007 Circles — the encrypted sub-boundary
//! primitive that sits *inside* a Realm. Each Circle carries its own
//! MLS group and a member list that MUST be a strict subset of its
//! parent Realm's membership; the soland reducer enforces the subset
//! invariant via `circle_member_must_be_realm_member`, and these pages
//! reflect the same constraint up front so operators see the rule
//! before they hit the server.
//!
//! Routes wired in `router.rs`:
//!
//! - `/circles`                      — list (per-Realm filter)
//! - `/circles/new`                  — create
//! - `/circles/:circle_id`           — detail + lifecycle actions
//! - `/circles/:circle_id/members`   — member editor
//! - `/circles/:circle_id/scope`     — MLS scope rotation surface
pub mod create;
pub mod list;
pub mod members;
pub mod scope;
pub mod show;
