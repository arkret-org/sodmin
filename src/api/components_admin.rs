//! HTTP client for the server-wide component registry admin describe
//! endpoint (Stream H', H'6).
//!
//! Endpoint:
//!
//! - `GET /api/admin/v1/components` — server-wide list of registered
//!   `cx.component.*` types, their cell_family, criticality, pinned spec
//!   version and loaded impl version. The page compares spec vs impl and
//!   surfaces a drift indicator on each row.

use crate::api::client::{api_client, build_url};
use crate::types::components::ComponentRegistryEntry;
use crate::utils::error::HttpError;

/// Fetch the full server-wide component registry. soland walks its
/// reducer registry and returns one entry per component_type the server
/// knows about, regardless of whether any Space has instances.
pub async fn list_components() -> Result<Vec<ComponentRegistryEntry>, HttpError> {
    let url = build_url("/api/admin/v1/components", &[])?;
    api_client(&url, "GET", None).await
}
