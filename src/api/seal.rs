//! HTTP client for the Move/Seal/Lattice admin endpoints exposed by
//! soland (Stream H', C10.F).
//!
//! Endpoint shape mirrors the canonical `/_soland/admin/...` admin surface
//! used by the rest of sodmin (see `api/spaces.rs`); the soland routes
//! land at:
//!
//! - `GET  /_soland/admin/realms/{realm_id}/notary`                   — describe current notary
//!   cell value (SDK-authoritative `NotaryValue` + envelope hints)
//! - `POST /_soland/admin/realms/{realm_id}/notary/reconfigure`       — submit reconfig Move (body
//!   is exactly the proposed `NotaryValue`)
//! - `GET  /_soland/admin/bottom`                                     — list ⊥ cells globally
//! - `POST /_soland/admin/realms/{realm_id}/bottom/{cell_id}/repair`  — submit repair Move
//! - `GET  /_soland/admin/realms/{realm_id}/seal-dag`                 — leaves+digests+state_root
//! - `POST /_soland/admin/realms/{realm_id}/seal-dag/compact`         — trigger compaction Seal
//!
//! The soland handlers translate the typed request bodies into real
//! Moves / Seals, sign them with the admin signer flow bound to the
//! bearer token, and POST onto the canonical Move / Seal pipelines.

use crate::api::client::{api_client, build_url, json_body};
use crate::types::seal::{
    BottomEntry, BottomRepairRequest, BottomRepairStrategy, CompactionOutcome, CompactionRequest,
    NotaryCellValue, NotaryValue, SealDagSnapshot, SubmitMoveOutcome,
};
use crate::utils::net::error::HttpError;

/// Fetch the current notary cell value for a Realm.
///
/// `GET /_soland/admin/realms/{realm_id}/notary`. soland projects the joined
/// `ck:cell:ck.component.notary.v1:<realm_id>` value plus the surrounding
/// hint fields (`revocation_freshness_window_ms`, `paused`).
pub async fn get_notary_value(realm_id: &str) -> Result<NotaryCellValue, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/notary",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

/// Submit a notary reconfiguration Move.
///
/// `POST /_soland/admin/realms/{realm_id}/notary/reconfigure`. The body is
/// exactly the SDK-authoritative `NotaryValue` wire shape (internal tag
/// `kind`, fields `did|k|n|members|primary|recovery_members`); soland
/// validates it via `NotaryValue::validate()`, builds the typed Move on the
/// server side, signs with the admin signer flow tied to the bearer token,
/// and submits onto the Move pipeline.
pub async fn submit_notary_reconfig(
    realm_id: &str,
    value: &NotaryValue,
) -> Result<SubmitMoveOutcome, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/notary/reconfigure",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "POST", Some(json_body(value)?)).await
}

/// List bottom entries across every Realm the admin can see — used by
/// the global "Bottom diagnostics" page in the sidebar (no per-Realm
/// pre-filter).
///
/// `GET /_soland/admin/bottom`. Each row carries its `realm_id` so the
/// renderer can link out.
pub async fn list_bottom_entries_global() -> Result<Vec<BottomEntry>, HttpError> {
    let url = build_url("/_soland/admin/bottom", &[])?;
    api_client(&url, "GET", None).await
}

/// Submit a "construct repair Move" for a single bottom cell.
///
/// `POST /_soland/admin/realms/{realm_id}/bottom/{cell_id}/repair`. The body
/// carries the typed `BottomRepairStrategy` — `head_in_winner` (the
/// admin picks one of the concurrent heads) or `manual` (free-form
/// effects array, used as the escape hatch for non-conflict bottoms).
pub async fn submit_bottom_repair(
    realm_id: &str,
    cell_id: &str,
    strategy: BottomRepairStrategy,
) -> Result<SubmitMoveOutcome, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/bottom/{}/repair",
        urlencoding::encode(realm_id),
        urlencoding::encode(cell_id),
    );
    let req = BottomRepairRequest {
        realm_id: realm_id.to_string(),
        cell_id: cell_id.to_string(),
        strategy,
    };
    api_client(&url, "POST", Some(json_body(&req)?)).await
}

/// Fetch the Seal DAG snapshot (leaves + covered event digests +
/// state_root + last compaction timestamp).
///
/// `GET /_soland/admin/realms/{realm_id}/seal-dag`.
pub async fn get_seal_dag(realm_id: &str) -> Result<SealDagSnapshot, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/seal-dag",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

/// Trigger a signed compaction Seal.
///
/// `POST /_soland/admin/realms/{realm_id}/seal-dag/compact`. soland's
/// handler folds pending Moves into a fresh compaction Seal and returns
/// the new seal id + state_root.
pub async fn trigger_compaction(realm_id: &str) -> Result<CompactionOutcome, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/seal-dag/compact",
        urlencoding::encode(realm_id)
    );
    let req = CompactionRequest { max_moves: None };
    api_client(&url, "POST", Some(json_body(&req)?)).await
}

#[cfg(test)]
mod tests {
    //! Pure-shape tests — these verify that the request bodies we POST
    //! match the wire shape soland's handlers expect, without needing a
    //! live HTTP loop.

    use cokret_core::Did;

    use super::*;
    use crate::types::seal::WinnerHead;

    #[test]
    fn reconfig_body_renders_sdk_threshold_shape() {
        let value = NotaryValue::Threshold {
            k: 2,
            n: 3,
            members: vec![
                Did::new("did:ck:a".to_owned()).unwrap(),
                Did::new("did:ck:b".to_owned()).unwrap(),
                Did::new("did:ck:c".to_owned()).unwrap(),
            ],
        };
        let body = serde_json::to_value(&value).unwrap();
        // Wire body is the SDK-authoritative internally tagged shape —
        // `kind` + `k`/`n`/`members`, no flat alias spellings.
        assert_eq!(body["kind"], "threshold");
        assert_eq!(body["k"], 2);
        assert_eq!(body["n"], 3);
        assert_eq!(body["members"].as_array().map(|a| a.len()), Some(3));
        assert!(body.get("threshold_k").is_none());
        assert!(body.get("threshold_dids").is_none());
        assert!(body.get("single_did").is_none());
        assert!(body.get("open_set_members").is_none());
    }

    #[test]
    fn repair_request_body_serializes_with_strategy_tag() {
        let req = BottomRepairRequest {
            realm_id: "ck:realm:demo".into(),
            cell_id: "ck:cell:ck.component.notary.v1:ck:space:demo".into(),
            strategy: BottomRepairStrategy::HeadInWinner {
                head: WinnerHead {
                    move_id: "sha256:aaaa".into(),
                    issuer: Some("did:ck:alice".into()),
                    hlc: None,
                    summary: None,
                },
            },
        };
        let s = serde_json::to_string(&req).unwrap();
        // Strategy MUST be tagged on the wire so the soland handler can
        // pattern-match without sniffing the rest of the body.
        assert!(s.contains("\"strategy\":\"head_in_winner\""));
        assert!(s.contains("\"move_id\":\"sha256:aaaa\""));
    }

    #[test]
    fn compaction_request_body_default_omits_max_moves() {
        let req = CompactionRequest { max_moves: None };
        let s = serde_json::to_string(&req).unwrap();
        assert!(!s.contains("max_moves"));
    }
}
