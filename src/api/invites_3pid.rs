//! HTTP client for the coauth third-party invite admin listing.

use crate::api::client::api_client;
use crate::types::{ListResponse, ThirdPartyInviteRow};
use crate::utils::error::HttpError;

pub async fn list_third_party_invites() -> Result<ListResponse<ThirdPartyInviteRow>, HttpError> {
    api_client("/_soland/admin/invites/3pid", "GET", None).await
}
