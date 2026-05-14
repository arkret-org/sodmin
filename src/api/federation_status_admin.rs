//! HTTP client for the soland federation status admin surface.
//!
//! Endpoint: `GET /api/admin/v1/federation/status` — per-Space
//! federation peers + last-anchor-pulled-at + outbound queue depth.
//! 404-tolerant on the client side.
//!
//! Row / health DTOs are sourced from `coauth_admin_types::federation_admin`.
//! The top-level envelope is the sodmin client's list-paging shape.

use coauth_admin_types::federation_admin::FederationStatusRow;
use serde::{Deserialize, Serialize};

use crate::api::client::{api_client, build_url};
use crate::utils::error::HttpError;

/// Top-level envelope returned by `GET /api/admin/v1/federation/status`.
/// soland MAY return either a flat list or a wrapped envelope; the API
/// client supports both shapes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FederationStatusEnvelope {
    #[serde(default)]
    pub data: Vec<FederationStatusRow>,
    #[serde(default)]
    pub generated_at: Option<String>,
}

pub async fn get_status(space_id: Option<&str>) -> Result<FederationStatusEnvelope, HttpError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = space_id.filter(|s| !s.is_empty()) {
        params.push(("space_id", s));
    }
    let url = build_url("/api/admin/v1/federation/status", &params)?;
    api_client(&url, "GET", None).await
}
