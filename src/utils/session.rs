//! Typed read-only views into the current admin session that lives in
//! `localStorage`. Pages and infrastructure should prefer these helpers
//! over hand-rolled `storage::get_item("...")` calls so the storage key
//! names ("access_token", "user_id", "coauth_public_url", …) only have
//! to be defined and spelled correctly in one place.
//!
//! The write path (login, logout, refresh, OAuth callback) is owned by
//! `crate::api::auth` — this module is intentionally read-only and
//! re-exports the small number of cached read helpers from there so the
//! caller can `use crate::utils::session::*` and get the full picture.

use crate::utils::storage;

pub use crate::api::auth::{cached_is_admin, is_authenticated, token_expiry_ms};

/// Snapshot of the currently signed-in admin's display attributes,
/// derived from the OIDC userinfo we cache after a successful OAuth
/// callback. All fields are independently optional so partially
/// populated sessions (e.g. only `user_id` known) still render.
#[derive(Debug, Clone, Default)]
pub struct CurrentUser {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

impl CurrentUser {
    /// True if any of the identity fields are populated. Used by pages
    /// like `NotAuthorizedPage` that show a "Signed in as ..." block
    /// only when there's at least one piece of identity to display.
    pub fn has_identity(&self) -> bool {
        self.id.is_some() || self.display_name.is_some()
    }
}

pub fn current_user() -> CurrentUser {
    CurrentUser {
        id: storage::get_item("user_id").filter(|v| !v.is_empty()),
        display_name: storage::get_item("user_display_name").filter(|v| !v.is_empty()),
        avatar_url: storage::get_item("user_avatar_url").filter(|v| !v.is_empty()),
    }
}

/// Bearer token used by `api::client::api_client` for the
/// `Authorization: Bearer ...` header. `None` means we have no
/// credentials and the next request should bounce to login.
pub fn access_token() -> Option<String> {
    storage::get_item("access_token")
}

/// Origin of the upstream coauth service we federate authentication
/// against. Populated by `pages::login` once `/config.json` has been
/// fetched.
pub fn coauth_public_url() -> Option<String> {
    storage::get_item("coauth_public_url")
}

/// Convenience for the sidebar / dashboard surfaces that hide the
/// "coauth" admin sections when the operator has not configured a
/// coauth public URL.
pub fn has_coauth() -> bool {
    coauth_public_url().is_some()
}
