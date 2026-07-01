//! coauth admin viewer (current operator identity).

use serde::{Deserialize, Serialize};

use crate::api::client::{api_client, NO_BODY};
use crate::utils::net::error::HttpError;

const VIEWER_PATH: &str = "/_cokret/self/account/viewer";

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
    api_client(VIEWER_PATH, "GET", NO_BODY).await
}
