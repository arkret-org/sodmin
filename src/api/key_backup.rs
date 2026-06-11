//! B-C key-backup admin surface — talks to the soland endpoints
//! shipped in P2 (aa76b91): `GET /_cokret/self/keys/backups?series_id=...`
//! plus the soland identity recovery extension endpoints.

use crate::api::client::{api_client, build_url};
use crate::types::{KeyBackupListResponse, ListResponse, RecoveryPolicy, RecoveryReceipt};
use crate::utils::net::error::HttpError;

// Recovery policy / receipt browse is a soland identity extension on the
// product surface (`/_soland/root/identity/*`); it is NOT a `/_cokret`
// protocol operation (the protocol surface only has describe/resolve/
// document/log/receipts/submit-did-operation/recovery-sessions).
const RECOVERY_POLICIES_PATH: &str = "/_soland/root/identity/recovery-policies";
const RECOVERY_POLICY_PATH: &str = "/_soland/root/identity/recovery-policy";
const RECOVERY_RECEIPTS_PATH: &str = "/_soland/root/identity/recovery-receipts";

/// `GET /_cokret/self/keys/backups?series_id=&backup_class=` — list backup
/// envelopes grouped by series. Empty `series_id` returns the per-series
/// frontier roll-up.
pub async fn list_backups(
    series_id: Option<&str>,
    backup_class: Option<&str>,
) -> Result<KeyBackupListResponse, HttpError> {
    let mut params: Vec<(&str, &str)> = Vec::with_capacity(2);
    if let Some(s) = series_id {
        params.push(("series_id", s));
    }
    if let Some(c) = backup_class {
        params.push(("backup_class", c));
    }
    let url = build_url("/_cokret/self/keys/backups", &params)?;
    api_client(&url, "GET", None).await
}

/// `GET /_soland/root/identity/recovery-policies` — current recovery policies.
pub async fn list_recovery_policies() -> Result<ListResponse<RecoveryPolicy>, HttpError> {
    api_client(RECOVERY_POLICIES_PATH, "GET", None).await
}

/// `POST /_soland/root/identity/recovery-policy` — upsert a policy's
/// lifecycle / KDF profile. soland exposes the mutation as a singular POST
/// (there is no per-id `PUT recovery-policies/{id}` route). The deep
/// validators (epoch hash monotonicity, KDF profile compat, lifecycle
/// transition rules) live server-side.
pub async fn upsert_recovery_policy(policy: &RecoveryPolicy) -> Result<RecoveryPolicy, HttpError> {
    let body =
        serde_json::to_string(policy).map_err(|e| HttpError::message(format!("serialize: {e}")))?;
    api_client(RECOVERY_POLICY_PATH, "POST", Some(body)).await
}

/// `GET /_soland/root/identity/recovery-receipts` — admin browse of issued
/// recovery receipts.
pub async fn list_recovery_receipts(
    session_id: Option<&str>,
) -> Result<ListResponse<RecoveryReceipt>, HttpError> {
    let mut params: Vec<(&str, &str)> = Vec::with_capacity(1);
    if let Some(s) = session_id {
        params.push(("session_id", s));
    }
    let url = build_url(RECOVERY_RECEIPTS_PATH, &params)?;
    api_client(&url, "GET", None).await
}
