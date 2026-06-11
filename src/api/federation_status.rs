//! HTTP client for the soland federation status admin surface.
//!
//! The per-Realm federation health endpoint is not currently wired by
//! soland. The page keeps the DTOs so the UI can render a clear local
//! "not wired" error without probing a false path.
//!
//! Row / health DTOs are sourced from `coauth_admin_types::federation_admin`.
//! The top-level envelope is the sodmin client's list-paging shape.

use coauth_admin_types::federation_admin::FederationStatusRow;
use serde::{Deserialize, Serialize};

use crate::utils::net::error::HttpError;

/// Top-level envelope expected once soland exposes federation health.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FederationStatusEnvelope {
    #[serde(default)]
    pub data: Vec<FederationStatusRow>,
    #[serde(default)]
    pub generated_at: Option<String>,
}

pub async fn get_status(realm_id: Option<&str>) -> Result<FederationStatusEnvelope, HttpError> {
    let _ = realm_id;
    Err(HttpError::message(
        "federation status endpoint is not wired",
    ))
}
