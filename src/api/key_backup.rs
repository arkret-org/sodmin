//! B-C key-backup admin surface — talks to the soland endpoints
//! deployment-local admin alias: `GET /_soland/admin/key-backups?series_id=...`
//! plus the soland identity recovery extension endpoints.

use arkret_identifiers::DidCoreId;
use arkret_models_crypto::{KeysBackupsList, RecoveryPolicySummary};

use crate::api::client::{NO_BODY, api_client, build_url};
use crate::types::ListResponse;
use crate::utils::net::error::HttpError;

// Recovery policy browse is a soland identity extension on the
// product surface (`/_soland/root/identity/*`); it is NOT a `/_arkret`
// protocol operation (the protocol surface only has describe/resolve/
// document/log/receipts/submit-did-operation/recovery-sessions).
const RECOVERY_POLICIES_PATH: &str = "/_soland/root/identity/recovery-policies";

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct RecoveryPoliciesEnvelope {
    #[serde(default)]
    policies: Vec<RecoveryPolicySummary>,
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    total: Option<u64>,
}

/// `GET /_soland/admin/key-backups?series_id=&backup_kind=` — list backup
/// envelopes grouped by series. Empty `series_id` returns the per-series
/// frontier roll-up.
pub async fn list_backups(
    series_id: Option<&str>,
    backup_kind: Option<&str>,
) -> Result<KeysBackupsList, HttpError> {
    let mut params: Vec<(&str, &str)> = Vec::with_capacity(2);
    if let Some(s) = series_id {
        params.push(("series_id", s));
    }
    if let Some(c) = backup_kind {
        params.push(("backup_kind", c));
    }
    let url = build_url("/_soland/admin/key-backups", &params)?;
    api_client(&url, "GET", NO_BODY).await
}

/// `GET /_soland/root/identity/recovery-policies` — current recovery policies.
pub async fn list_recovery_policies(
    principal_id: Option<&DidCoreId>,
) -> Result<ListResponse<RecoveryPolicySummary>, HttpError> {
    let mut params: Vec<(&str, &str)> = Vec::with_capacity(1);
    if let Some(principal_id) = principal_id {
        params.push(("principal_id", principal_id.as_str()));
    }
    let url = build_url(RECOVERY_POLICIES_PATH, &params)?;
    let resp: RecoveryPoliciesEnvelope = api_client(&url, "GET", NO_BODY).await?;
    Ok(ListResponse {
        data: resp.policies,
        total: resp.total,
        next_cursor: resp.next_cursor,
    })
}

// NOTE: `POST /_soland/root/identity/recovery-policy` requires the full
// `ak.schema.recovery_policy.v1` object carrying a principal-signed
// `auth_data` transcript (REC-1) — the admin UI cannot sign on the
// principal's behalf, so sodmin exposes the recovery surface read-only.
