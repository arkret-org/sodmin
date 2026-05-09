//! Applets admin pages.
//!
//! - [`list`] — legacy applet registration list (existing yougen-side
//!   applets the operator owns).
//! - [`admin`] — Round 25, F1 — soland-side applets admin with
//!   per-row Approve / Revoke. Light scaffolding; full UI in round 26.

pub mod admin;
pub mod list;

pub use list::AppletList;
