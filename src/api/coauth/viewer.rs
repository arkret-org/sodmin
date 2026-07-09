//! coauth admin viewer (current operator identity).

use crate::api::client::{NO_BODY, api_client};
use crate::utils::net::error::HttpError;

const VIEWER_PATH: &str = "/_arkret/self/account/viewer";

pub type CoauthViewer = arkret_core::models::AccountView;

pub async fn get_viewer() -> Result<CoauthViewer, HttpError> {
    api_client(VIEWER_PATH, "GET", NO_BODY).await
}
