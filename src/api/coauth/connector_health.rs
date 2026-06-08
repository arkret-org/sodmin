//! coauth admin connector health.

pub use coauth_admin_types::ConnectorHealthRow as CoauthConnectorHealth;

use crate::api::client::api_client;
use crate::api::openapi_contract::coauth as coauth_paths;
use crate::utils::net::error::HttpError;

pub async fn get_connector_health() -> Result<Vec<CoauthConnectorHealth>, HttpError> {
    let resp: coauth_admin_types::ConnectorHealthOutcome =
        api_client(coauth_paths::CONNECTOR_HEALTH, "GET", None).await?;
    Ok(resp.providers)
}
