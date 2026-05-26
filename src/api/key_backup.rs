//! B-C key-backup admin surface — talks to the soland endpoints
//! shipped in P2 (aa76b91): `GET /api/v1/keys/backups?series_id=...`
//! plus the recovery-policy and recovery-receipt typed-id endpoints.

use crate::api::client::{api_client, build_url};
use crate::types::{KeyBackupSeries, ListResponse, RecoveryPolicy, RecoveryReceipt};
use crate::utils::error::HttpError;

/// `GET /api/v1/keys/backups?series_id=&backup_class=` — list backup
/// envelopes grouped by series. Empty `series_id` returns the per-series
/// frontier roll-up.
pub async fn list_backups(
    series_id: Option<&str>,
    backup_class: Option<&str>,
) -> Result<ListResponse<KeyBackupSeries>, HttpError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = series_id {
        params.push(("series_id", s));
    }
    if let Some(c) = backup_class {
        params.push(("backup_class", c));
    }
    let url = build_url("/api/v1/keys/backups", &params)?;
    api_client(&url, "GET", None).await
}

/// `GET /api/v1/keys/recovery-policies` — current recovery policies.
pub async fn list_recovery_policies() -> Result<ListResponse<RecoveryPolicy>, HttpError> {
    api_client("/api/v1/keys/recovery-policies", "GET", None).await
}

/// `PUT /api/v1/keys/recovery-policies/{id}` — edit a policy's
/// lifecycle / KDF profile. The deep validators (epoch hash
/// monotonicity, KDF profile compat, lifecycle transition rules) are
/// `TODO(P3-impl)` and live server-side.
pub async fn upsert_recovery_policy(policy: &RecoveryPolicy) -> Result<RecoveryPolicy, HttpError> {
    let url = format!(
        "/api/v1/keys/recovery-policies/{}",
        urlencoding::encode(&policy.policy_id)
    );
    let body = serde_json::to_string(policy).map_err(|e| HttpError {
        message: format!("serialize: {e}"),
        status: 0,
        body: None,
        request_id: None,
        retry_after_ms: None,
    })?;
    api_client(&url, "PUT", Some(body)).await
}

/// `GET /api/v1/keys/recovery-receipts` — admin browse of issued
/// recovery receipts.
pub async fn list_recovery_receipts(
    session_id: Option<&str>,
) -> Result<ListResponse<RecoveryReceipt>, HttpError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = session_id {
        params.push(("session_id", s));
    }
    let url = build_url("/api/v1/keys/recovery-receipts", &params)?;
    api_client(&url, "GET", None).await
}
