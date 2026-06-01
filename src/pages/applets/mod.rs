//! Applets admin pages.
//!
//! - [`list`] — applet registration list (existing yougen-side applets the operator owns).
//! - [`admin`] — per-row Approve / Revoke.

pub mod admin;
pub mod list;

pub use list::AppletList;
