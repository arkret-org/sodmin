//! coauth admin connector health.

pub use coauth_admin_types::ConnectorHealthRow as CoauthConnectorHealth;

use crate::api::client::{api_client, NO_BODY};
use crate::utils::net::error::HttpError;

const CONNECTOR_HEALTH_PATH: &str = "/_coauth/admin/connector-health";

pub async fn get_connector_health() -> Result<Vec<CoauthConnectorHealth>, HttpError> {
    let resp: coauth_admin_types::ConnectorHealthOutcome =
        api_client(CONNECTOR_HEALTH_PATH, "GET", NO_BODY).await?;
    Ok(resp.providers)
}
