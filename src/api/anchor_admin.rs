//! HTTP client for the Move/Anchor/Lattice admin endpoints exposed by
//! soland (Stream H', C10.F).
//!
//! Endpoint shape mirrors the canonical `/_soland/admin/...` admin surface
//! used by the rest of sodmin (see `api/spaces.rs`); the soland routes
//! land at:
//!
//! - `GET  /_soland/admin/spaces/{id}/anchorer`                 — describe current anchorer cell
//!   value
//! - `POST /_soland/admin/spaces/{id}/anchorer/reconfigure`     — submit reconfig Move
//! - `GET  /_soland/admin/spaces/{id}/bottom`                   — list ⊥ cells in this Space
//! - `POST /_soland/admin/spaces/{id}/bottom/{cell_id}/repair`  — submit repair Move
//! - `GET  /_soland/admin/spaces/{id}/anchor-dag`               — leaves+frontier+state_root
//! - `POST /_soland/admin/spaces/{id}/anchor-dag/compact`       — trigger compaction Anchor
//!
//! The soland handlers translate the typed request bodies into real
//! Moves / Anchors, sign them with the principal-server's anchorer key
//! (or, for the reconfigure endpoint, route through the admin's signer
//! flow with the bearer token from the `Authorization` header), and
//! POST onto the canonical Move / Anchor pipelines.

use crate::api::client::{api_client, build_url};
use crate::types::anchor::{
    AnchorDagSnapshot, AnchorerReconfigRequest, AnchorerValue, BottomEntry, BottomRepairRequest,
    BottomRepairStrategy, CompactionRequest, SignAnchorResponse, SubmitMoveResponse,
};
use crate::utils::net::error::HttpError;

/// Fetch the current anchorer cell value for a Space.
///
/// `GET /_soland/admin/spaces/{id}/anchorer`. soland projects the joined
/// `ck:cell:ck.component.anchorer.v1:<space>` value plus the surrounding
/// hint fields (`max_anchor_staleness_ms`, `paused`).
pub async fn get_anchorer_value(realm_id: &str) -> Result<AnchorerValue, HttpError> {
    let url = format!(
        "/_soland/admin/spaces/{}/anchorer",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

/// Submit an anchorer reconfiguration Move.
///
/// `POST /_soland/admin/spaces/{id}/anchorer/reconfigure`. The body shape
/// is `{ kind, single_did?, threshold_k?, threshold_n?, threshold_dids?, ... }`
/// (see `AnchorerReconfigRequest::to_reconfigure_body`); soland builds
/// the typed Move on the server side, signs with the admin's key (or
/// routes through the admin signer flow tied to the bearer token), and
/// posts onto `/_cokret/peer/moves`.
pub async fn submit_anchorer_reconfig(
    req: &AnchorerReconfigRequest,
) -> Result<SubmitMoveResponse, HttpError> {
    let url = format!(
        "/_soland/admin/spaces/{}/anchorer/reconfigure",
        urlencoding::encode(&req.realm_id)
    );
    let body = req.to_reconfigure_body();
    api_client(
        &url,
        "POST",
        Some(serde_json::to_string(&body).unwrap_or_default()),
    )
    .await
}

/// List bottom entries across every Space the admin can see — used by
/// the global "Bottom diagnostics" page in the sidebar (no per-Space
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
/// `POST /_soland/admin/spaces/{id}/bottom/{cell_id}/repair`. The body
/// carries the typed `BottomRepairStrategy` — `head_in_winner` (the
/// admin picks one of the concurrent heads) or `manual` (free-form
/// effects array, used as the escape hatch for non-conflict bottoms).
pub async fn submit_bottom_repair(
    realm_id: &str,
    cell_id: &str,
    strategy: BottomRepairStrategy,
) -> Result<SubmitMoveResponse, HttpError> {
    let url = format!(
        "/_soland/admin/spaces/{}/bottom/{}/repair",
        urlencoding::encode(realm_id),
        urlencoding::encode(cell_id),
    );
    let req = BottomRepairRequest {
        realm_id: realm_id.to_string(),
        cell_id: cell_id.to_string(),
        strategy,
    };
    api_client(
        &url,
        "POST",
        Some(serde_json::to_string(&req).unwrap_or_default()),
    )
    .await
}

/// Fetch the Anchor DAG snapshot (leaves + frontier + state_root + last
/// compaction timestamp).
///
/// `GET /_soland/admin/spaces/{id}/anchor-dag`.
pub async fn get_anchor_dag(realm_id: &str) -> Result<AnchorDagSnapshot, HttpError> {
    let url = format!(
        "/_soland/admin/spaces/{}/anchor-dag",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

/// Trigger a signed compaction Anchor.
///
/// `POST /_soland/admin/spaces/{id}/anchor-dag/compact`. soland's handler
/// is the admin-facing entry point onto `ck.admin.anchors.sign`; it folds
/// up to `max_moves` moves into a fresh compaction Anchor and returns
/// the new anchor id + state_root.
pub async fn trigger_compaction(realm_id: &str) -> Result<SignAnchorResponse, HttpError> {
    let url = format!(
        "/_soland/admin/spaces/{}/anchor-dag/compact",
        urlencoding::encode(realm_id)
    );
    let req = CompactionRequest {
        realm_id: realm_id.to_string(),
        max_moves: None,
    };
    api_client(
        &url,
        "POST",
        Some(serde_json::to_string(&req).unwrap_or_default()),
    )
    .await
}

#[cfg(test)]
mod tests {
    //! Pure-shape tests — these verify that the request bodies we POST
    //! match the wire shape soland's handlers expect, without needing a
    //! live HTTP loop.

    use super::*;
    use crate::types::anchor::WinnerHead;

    #[test]
    fn reconfig_request_body_renders_threshold_shape() {
        let req = AnchorerReconfigRequest {
            realm_id: "ck:space:0196419b-0000-7000-8000-000000000000".into(),
            kind: "threshold".into(),
            threshold_k: Some(2),
            threshold_n: Some(3),
            threshold_dids: vec!["did:ck:a".into(), "did:ck:b".into(), "did:ck:c".into()],
            ..Default::default()
        };
        let body = req.to_reconfigure_body();
        // Wire body must include kind+k+n+dids and exclude unrelated
        // shape fields so the soland handler doesn't see ambiguous input.
        assert_eq!(body["kind"], "threshold");
        assert_eq!(body["threshold_k"], 2);
        assert_eq!(body["threshold_n"], 3);
        assert_eq!(
            body["threshold_dids"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0),
            3
        );
        assert!(body.get("single_did").is_none());
        assert!(body.get("open_set_members").is_none());
    }

    #[test]
    fn repair_request_body_serializes_with_strategy_tag() {
        let req = BottomRepairRequest {
            realm_id: "ck:space:demo".into(),
            cell_id: "ck:cell:ck.component.anchorer.v1:ck:space:demo".into(),
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
        let req = CompactionRequest {
            realm_id: "ck:space:demo".into(),
            max_moves: None,
        };
        let s = serde_json::to_string(&req).unwrap();
        assert!(s.contains("\"realm_id\":\"ck:space:demo\""));
        assert!(!s.contains("max_moves"));
    }
}
