//! B-C key-backup admin surface — talks to the soland endpoints
//! shipped in P2 (aa76b91): `GET /_cokret/self/keys/backups?series_id=...`
//! plus the soland identity recovery extension endpoints.

use cokret_core::model::KeysBackupsList;

use crate::api::client::{api_client, build_url};
use crate::types::{ListResponse, RecoveryPolicySummary, RecoveryReceiptSummary};
use crate::utils::net::error::HttpError;

// Recovery policy / receipt browse is a soland identity extension on the
// product surface (`/_soland/root/identity/*`); it is NOT a `/_cokret`
// protocol operation (the protocol surface only has describe/resolve/
// document/log/receipts/submit-did-operation/recovery-sessions).
const RECOVERY_POLICIES_PATH: &str = "/_soland/root/identity/recovery-policies";
const RECOVERY_RECEIPTS_PATH: &str = "/_soland/root/identity/recovery-receipts";

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct RecoveryPoliciesEnvelope {
    #[serde(default)]
    policies: Vec<RecoveryPolicySummary>,
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    total: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct RecoveryReceiptsEnvelope {
    #[serde(default)]
    receipts: Vec<RecoveryReceiptSummary>,
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    total: Option<u64>,
}

/// `GET /_cokret/self/keys/backups?series_id=&backup_class=` — list backup
/// envelopes grouped by series. Empty `series_id` returns the per-series
/// frontier roll-up.
pub async fn list_backups(
    series_id: Option<&str>,
    backup_class: Option<&str>,
) -> Result<KeysBackupsList, HttpError> {
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
pub async fn list_recovery_policies(
    principal_id: Option<&str>,
) -> Result<ListResponse<RecoveryPolicySummary>, HttpError> {
    let mut params: Vec<(&str, &str)> = Vec::with_capacity(1);
    if let Some(principal_id) = principal_id.filter(|s| !s.is_empty()) {
        params.push(("principal_id", principal_id));
    }
    let url = build_url(RECOVERY_POLICIES_PATH, &params)?;
    let resp: RecoveryPoliciesEnvelope = api_client(&url, "GET", None).await?;
    Ok(ListResponse {
        data: resp.policies,
        total: resp.total,
        next_cursor: resp.next_cursor,
    })
}

// NOTE: `POST /_soland/root/identity/recovery-policy` requires the full
// `ck.schema.recovery_policy.v1` object carrying a principal-signed
// `auth_data` transcript (REC-1) — the admin UI cannot sign on the
// principal's behalf, so sodmin exposes the recovery surface read-only.

/// `GET /_soland/root/identity/recovery-receipts` — admin browse of issued
/// recovery receipts.
pub async fn list_recovery_receipts(
    session_id: Option<&str>,
    principal_id: Option<&str>,
) -> Result<ListResponse<RecoveryReceiptSummary>, HttpError> {
    let mut params: Vec<(&str, &str)> = Vec::with_capacity(2);
    if let Some(s) = session_id {
        params.push(("session_id", s));
    }
    if let Some(principal_id) = principal_id.filter(|s| !s.is_empty()) {
        params.push(("principal_id", principal_id));
    }
    let url = build_url(RECOVERY_RECEIPTS_PATH, &params)?;
    let resp: RecoveryReceiptsEnvelope = api_client(&url, "GET", None).await?;
    Ok(ListResponse {
        data: resp.receipts,
        total: resp.total,
        next_cursor: resp.next_cursor,
    })
}
