//! HTTP client for the server-wide component registry admin describe
//! endpoint (Stream H', H'6).
//!
//! Endpoints:
//!
//! - `GET /_soland/admin/components` — server-wide list of registered `cx.component.*` types, their
//!   cell_family, criticality, pinned spec version and loaded impl version. The page compares spec
//!   vs impl and surfaces a drift indicator on each row.
//! - `POST /_soland/admin/components/{type}/refresh` — admin override for drifted components: ask
//!   the server to re-load the impl from the pinned spec version (clears stale caches, re-imports
//!   the reducer bundle, etc.). soland is expected to short-circuit when there's no drift; on 404
//!   the UI surfaces a "not yet wired" toast (see `pages/spaces/components.rs`).

use crate::api::client::{api_client, build_url};
use crate::types::components::{ComponentRefreshResponse, ComponentRegistryEntry};
use crate::utils::error::HttpError;

/// Fetch the full server-wide component registry. soland walks its
/// reducer registry and returns one entry per component_type the server
/// knows about, regardless of whether any Space has instances.
pub async fn list_components() -> Result<Vec<ComponentRegistryEntry>, HttpError> {
    let url = build_url("/_soland/admin/components", &[])?;
    api_client(&url, "GET", None).await
}

/// Ask the server to refresh a drifted component impl from its pinned
/// spec version. soland builds the canonical bundle URL for
/// `component_type` at `spec_version`, re-imports the reducer code, and
/// flips the registry entry's `impl_version` to match.
///
/// Idempotent on the server side; the same `Idempotency-Key` from the
/// admin client collapses retries.
pub async fn refresh(component_type: &str) -> Result<ComponentRefreshResponse, HttpError> {
    let url = format!(
        "/_soland/admin/components/{}/refresh",
        urlencoding::encode(component_type)
    );
    // Empty JSON body — the component type is in the path. The server
    // doesn't need any other parameters; the spec_version comes from the
    // server's own pinned manifest.
    api_client(&url, "POST", Some("{}".to_string())).await
}
