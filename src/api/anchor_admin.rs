//! Stub HTTP client for the Move/Anchor/Lattice admin endpoints.
//!
//! The real wire calls land later — soland exposes
//! `POST /api/v1/admin/anchors/sign`, `POST /api/v1/moves`, and
//! `POST /api/v1/anchors`, plus read-side describe endpoints we don't have
//! generated bindings for yet. Until then, these helpers return canned data
//! so the new admin pages can render and exercise their action paths.
//!
//! TODO(soland-admin-api): replace the stub responses with real HTTP fetches
//! against soland once the read-side describe endpoints (anchorer cell value,
//! bottom diagnostics, anchor DAG snapshot) are stabilised.

use crate::types::anchor::{
    AnchorDagSnapshot, AnchorLeaf, AnchorerReconfigRequest, AnchorerValue, BottomEntry,
    SignAnchorResponse, SubmitMoveResponse,
};
use crate::utils::error::HttpError;

/// Stub: fetch the anchorer cell value for a space.
///
/// TODO(soland-admin-api): wire to `GET /api/v1/spaces/{id}/anchorer`
/// (or whichever describe endpoint soland MAL-15 lands).
pub async fn get_anchorer_value(space_id: &str) -> Result<AnchorerValue, HttpError> {
    // Hardcoded sample for UI scaffolding.
    let _ = space_id;
    Ok(AnchorerValue {
        kind_raw: "threshold".into(),
        threshold_k: Some(2),
        threshold_n: Some(3),
        threshold_dids: vec![
            "did:cx:anchorer-a".into(),
            "did:cx:anchorer-b".into(),
            "did:cx:anchorer-c".into(),
        ],
        max_anchor_staleness_ms: Some(60_000),
        paused: false,
        ..Default::default()
    })
}

/// Stub: submit an anchorer reconfiguration Move.
///
/// TODO(soland-admin-api): build a real Move payload from the request and
/// POST to soland's `/api/v1/moves`. The current stub just echoes a fake
/// move_id without contacting the backend.
pub async fn submit_anchorer_reconfig(
    req: &AnchorerReconfigRequest,
) -> Result<SubmitMoveResponse, HttpError> {
    Ok(SubmitMoveResponse {
        move_id: format!("stub-move-{}-{}", req.space_id, req.kind),
        accepted: true,
        reason: None,
    })
}

/// Stub: list all cells currently in Bottom state across visible Spaces.
///
/// TODO(soland-admin-api): wire to soland's bottom-diagnostics describe
/// endpoint when MAL-5 lands.
pub async fn list_bottom_entries() -> Result<Vec<BottomEntry>, HttpError> {
    Ok(vec![
        BottomEntry {
            space_id: "space:demo-1".into(),
            cell_id: "cx:cell:cx.component.anchorer.v1:space:demo-1".into(),
            kind: "anchorer_split".into(),
            move_ids: vec!["move:abc".into(), "move:def".into()],
            details: Some("Two disjoint anchorer reconfig branches detected.".into()),
            detected_at: Some("2026-05-09T03:21:00Z".into()),
        },
        BottomEntry {
            space_id: "space:demo-2".into(),
            cell_id: "cx:cell:cx.component.profile.v1:space:demo-2".into(),
            kind: "conflict".into(),
            move_ids: vec!["move:xyz".into()],
            details: Some("LWW register received concurrent writes that cannot join.".into()),
            detected_at: Some("2026-05-09T03:55:00Z".into()),
        },
    ])
}

/// Stub: submit a "construct repair Move" for a bottom entry.
///
/// TODO(soland-admin-api): the admin UI builds a repair payload (chosen
/// branch / override) and POSTs to `/api/v1/moves`. Today we just return a
/// fake move_id.
pub async fn submit_bottom_repair(
    space_id: &str,
    cell_id: &str,
) -> Result<SubmitMoveResponse, HttpError> {
    Ok(SubmitMoveResponse {
        move_id: format!("stub-repair-{}-{}", space_id, cell_id),
        accepted: true,
        reason: None,
    })
}

/// Stub: fetch Anchor DAG snapshot (leaves + frontier + state_root).
///
/// TODO(soland-admin-api): wire to soland's anchor-dag describe endpoint.
pub async fn get_anchor_dag(space_id: &str) -> Result<AnchorDagSnapshot, HttpError> {
    Ok(AnchorDagSnapshot {
        space_id: space_id.to_string(),
        leaves: vec![
            AnchorLeaf {
                anchor_id: "anchor:001".into(),
                state_root: Some("sha256:aaaa".into()),
                move_count: 42,
                created_at: Some("2026-05-09T01:00:00Z".into()),
                signers: vec!["did:cx:anchorer-a".into(), "did:cx:anchorer-b".into()],
                is_compaction: false,
            },
            AnchorLeaf {
                anchor_id: "anchor:002".into(),
                state_root: Some("sha256:bbbb".into()),
                move_count: 17,
                created_at: Some("2026-05-09T02:30:00Z".into()),
                signers: vec!["did:cx:anchorer-b".into(), "did:cx:anchorer-c".into()],
                is_compaction: false,
            },
        ],
        frontier: vec!["anchor:001".into(), "anchor:002".into()],
        state_root: Some("sha256:bbbb".into()),
        last_compaction_at: Some("2026-05-08T22:00:00Z".into()),
    })
}

/// Stub: trigger signed compaction Anchor.
///
/// TODO(soland-admin-api): replace with real call to
/// `POST /api/v1/admin/anchors/sign` (`cx.admin.anchors.sign`).
pub async fn trigger_compaction(space_id: &str) -> Result<SignAnchorResponse, HttpError> {
    Ok(SignAnchorResponse {
        anchor_id: format!("stub-compaction-{}", space_id),
        state_root: Some("sha256:cccc".into()),
        move_count: 0,
    })
}
