//! coauth admin notification channels and templates.

pub use coauth_admin_types::{
    NotificationChannelStatus as CoauthNotificationChannel,
    NotificationTemplateEntry as CoauthNotificationTemplate,
};

use crate::api::client::api_client;
use crate::api::openapi_contract::coauth as coauth_paths;
use crate::utils::net::error::HttpError;

pub async fn list_notification_channels() -> Result<Vec<CoauthNotificationChannel>, HttpError> {
    let resp: coauth_admin_types::NotificationChannelsOutcome =
        api_client(coauth_paths::NOTIFICATION_CHANNELS, "GET", None).await?;
    Ok(resp.channels)
}

/// Returns the static catalog of known notification template keys. The
/// backend endpoint is non-paginated — it returns the full known-keys
/// list every call — so the sodmin UI now flat-lists the entries
/// instead of pretending there's a paging cursor.
pub async fn list_notification_templates() -> Result<Vec<CoauthNotificationTemplate>, HttpError> {
    let resp: coauth_admin_types::NotificationTemplatesOutcome =
        api_client(coauth_paths::NOTIFICATION_TEMPLATES, "GET", None).await?;
    Ok(resp.templates)
}
