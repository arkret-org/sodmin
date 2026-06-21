//! coauth admin notification channels and templates.

pub use coauth_admin_types::{
    NotificationChannelStatus as CoauthNotificationChannel,
    NotificationTemplateEntry as CoauthNotificationTemplate,
};

use crate::api::client::api_client;
use crate::utils::net::error::HttpError;

const NOTIFICATION_CHANNELS_PATH: &str = "/_coauth/admin/notification-channels";
const NOTIFICATION_TEMPLATES_PATH: &str = "/_coauth/admin/notification-templates";

pub async fn list_notification_channels() -> Result<Vec<CoauthNotificationChannel>, HttpError> {
    let resp: coauth_admin_types::NotificationChannelsOutcome =
        api_client(NOTIFICATION_CHANNELS_PATH, "GET", None).await?;
    Ok(resp.channels)
}

/// Returns the static catalog of known notification template keys. The
/// backend endpoint is non-paginated — it returns the full known-keys
/// list every call — so the sodmin UI now flat-lists the entries
/// instead of pretending there's a paging cursor.
pub async fn list_notification_templates() -> Result<Vec<CoauthNotificationTemplate>, HttpError> {
    let resp: coauth_admin_types::NotificationTemplatesOutcome =
        api_client(NOTIFICATION_TEMPLATES_PATH, "GET", None).await?;
    Ok(resp.templates)
}
