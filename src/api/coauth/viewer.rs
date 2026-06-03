//! coauth admin viewer (current operator identity).

use serde::{Deserialize, Serialize};

use crate::api::client::api_client;
use crate::api::openapi_contract::coauth as coauth_paths;
use crate::utils::net::error::HttpError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthViewer {
    #[serde(default)]
    pub sub: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
}

pub async fn get_viewer() -> Result<CoauthViewer, HttpError> {
    api_client(coauth_paths::VIEWER, "GET", None).await
}
