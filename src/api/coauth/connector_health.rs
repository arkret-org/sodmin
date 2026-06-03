//! coauth admin connector health.

use crate::api::client::api_client;
use crate::api::openapi_contract::coauth as coauth_paths;
use crate::utils::net::error::HttpError;

pub use coauth_admin_types::ConnectorHealthRow as CoauthConnectorHealth;

pub async fn get_connector_health() -> Result<Vec<CoauthConnectorHealth>, HttpError> {
    let resp: coauth_admin_types::ConnectorHealthResponse =
        api_client(coauth_paths::CONNECTOR_HEALTH, "GET", None).await?;
    Ok(resp.providers)
}
