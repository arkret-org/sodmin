//! Typed read-only views into the current admin session (Round 25, S5
//! token hardening).
//!
//! Round 25 replaces the localStorage token with an httpOnly cookie
//! issued by coauth's `/oauth2/token` endpoint. The bearer token is
//! not readable from JS (cookie has `HttpOnly` + `Secure` +
//! `SameSite=Strict`), so this module no longer exposes an
//! `access_token()` accessor — the API client sends the cookie
//! automatically via `credentials: "include"`. What we do still keep
//! in localStorage is a *non-secret* `session_active=1` marker so the
//! UI can decide whether to render the login page vs. the
//! authenticated layout. The marker carries no entropy and is safe to
//! read from JS.
//!
//! See `api::auth` for the write path (login / logout / refresh / OAuth
//! callback). This module is intentionally read-only and re-exports the
//! small number of cached read helpers from there so callers can
//! `use crate::utils::session::*` and get the full picture.

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

/// Storage keys for the session-derived bridge + scope hints written by
/// the OAuth callback (or by `pages::login` after `/api/v1/server/describe`
/// resolves). The sidebar reads these to decide which nav groups to show.
const ACTIVE_BRIDGES_KEY: &str = "session_active_bridges";
const ADMIN_SCOPE_KEY: &str = "session_admin_scope";

/// Names of the bridge contracts the sidebar knows how to gate. These are
/// soft strings — anything missing from `active_bridges` simply hides the
/// corresponding group; unknown extras are ignored.
pub mod bridge {
    /// soland Move/Anchor/Lattice integration. Gates spaces/anchorer/
    /// anchor-dag/bottom and the rest of the Stream H' surface.
    pub const SOLAND: &str = "soland";

    /// coauth account/admin integration. Gates the entire coauth section.
    pub const COAUTH: &str = "coauth";

    /// floria push gateway — currently no admin pages, reserved.
    pub const FLORIA: &str = "floria";
}

/// Names of the admin scope buckets the sidebar gates pages against. A
/// missing entry hides the page entirely (unless the user has the
/// catch-all `urn:contrix:admin:*`).
pub mod scope {
    pub const WILDCARD: &str = "urn:contrix:admin:*";
    pub const IDENTITY: &str = "urn:contrix:admin:identity";
    pub const MODERATION: &str = "urn:contrix:admin:moderation";
    pub const INFRASTRUCTURE: &str = "urn:contrix:admin:infrastructure";
    pub const SERVER_OPS: &str = "urn:contrix:admin:server_ops";
    pub const ANCHOR: &str = "urn:contrix:admin:anchor";
    pub const COAUTH: &str = "urn:coauth:admin";
}

/// Parse a comma-separated storage value into a sorted, lower-cased
/// `Vec<String>`. Empty / missing values produce an empty vec.
fn parse_csv(value: Option<String>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Active bridge contracts as known by this admin session. The default
/// (no storage key set) auto-derives from what the deployment actually
/// has wired: the `coauth` bridge is on iff `coauth_public_url` is set,
/// and `soland` is assumed on (sodmin's whole reason to exist).
///
/// `pages::oauth_callback` / login may overwrite this with the bridges
/// the server actually advertised in `/api/v1/server/describe`.
pub fn active_bridges() -> Vec<String> {
    let stored = parse_csv(storage::get_item(ACTIVE_BRIDGES_KEY));
    if !stored.is_empty() {
        return stored;
    }
    let mut auto = vec![bridge::SOLAND.to_string()];
    if has_coauth() {
        auto.push(bridge::COAUTH.to_string());
    }
    auto
}

pub fn set_active_bridges(bridges: &[&str]) {
    storage::set_item(ACTIVE_BRIDGES_KEY, &bridges.join(","));
}

/// True if this admin session has the bridge enabled (or no override is
/// stored — fallback path for unprovisioned deployments).
pub fn has_bridge(name: &str) -> bool {
    active_bridges().iter().any(|b| b == name)
}

/// Admin scope set as known by this session. Default (no storage key) is
/// the wildcard scope so an unprovisioned deployment still sees every
/// page; once the session writer populates `session_admin_scope` the
/// sidebar tightens to the explicit subset.
pub fn admin_scope() -> Vec<String> {
    let stored = parse_csv(storage::get_item(ADMIN_SCOPE_KEY));
    if stored.is_empty() {
        return vec![scope::WILDCARD.to_string()];
    }
    stored
}

pub fn set_admin_scope(scopes: &[&str]) {
    storage::set_item(ADMIN_SCOPE_KEY, &scopes.join(","));
}

/// True if the current session's admin scope set covers `needed` (either
/// directly or via the catch-all wildcard).
pub fn has_scope(needed: &str) -> bool {
    let scopes = admin_scope();
    scopes.iter().any(|s| s == scope::WILDCARD || s == needed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_filters_empty() {
        assert_eq!(parse_csv(None), Vec::<String>::new());
        assert_eq!(parse_csv(Some(String::new())), Vec::<String>::new());
        assert_eq!(
            parse_csv(Some("a, b ,, c".to_string())),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }
}
