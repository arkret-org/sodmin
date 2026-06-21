//! HTTP client for the Notary / Seal / Bottom admin endpoints exposed by
//! soland.

use crate::api::client::{api_client, build_url, json_body};
use crate::types::seal::{
    BottomEntry, BottomRepairRequestBody, BottomRepairStrategy, CompactionOutcome,
    CompactionRequestBody, NotaryReconfigRequestBody, NotaryValue, SealDagSnapshot,
    SubmitControlMoveOutcome,
};
use crate::utils::net::error::HttpError;

pub async fn get_notary_value(realm_id: &str) -> Result<NotaryValue, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/notary",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

pub async fn submit_notary_reconfig(
    req: &NotaryReconfigRequestBody,
) -> Result<SubmitControlMoveOutcome, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/notary/reconfigure",
        urlencoding::encode(&req.realm_id)
    );
    let body = req.to_reconfigure_body();
    api_client(&url, "POST", Some(json_body(&body)?)).await
}

pub async fn list_bottom_entries_global() -> Result<Vec<BottomEntry>, HttpError> {
    let url = build_url("/_soland/admin/bottom", &[])?;
    api_client(&url, "GET", None).await
}

pub async fn submit_bottom_repair(
    realm_id: &str,
    cell_id: &str,
    strategy: BottomRepairStrategy,
) -> Result<SubmitControlMoveOutcome, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/bottom/{}/repair",
        urlencoding::encode(realm_id),
        urlencoding::encode(cell_id),
    );
    let req = BottomRepairRequestBody { strategy };
    api_client(&url, "POST", Some(json_body(&req)?)).await
}

pub async fn get_seal_dag(realm_id: &str) -> Result<SealDagSnapshot, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/seal-dag",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

pub async fn trigger_compaction(realm_id: &str) -> Result<CompactionOutcome, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/seal-dag/compact",
        urlencoding::encode(realm_id)
    );
    let req = CompactionRequestBody {
        realm_id: realm_id.to_string(),
        max_control_moves: None,
    };
    api_client(&url, "POST", Some(json_body(&req)?)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::seal::BottomCandidateHead;

    #[test]
    fn reconfig_request_body_renders_threshold_shape() {
        let req = NotaryReconfigRequestBody {
            realm_id: "ck:realm:0196419b-0000-7000-8000-000000000000".into(),
            kind: "threshold".into(),
            threshold_k: Some(2),
            threshold_n: Some(3),
            threshold_dids: vec![
                "did:web:a.example".into(),
                "did:web:b.example".into(),
                "did:web:c.example".into(),
            ],
            ..Default::default()
        };
        let body = req.to_reconfigure_body();

        assert_eq!(body["kind"], "threshold");
        assert_eq!(body["threshold_k"], 2);
        assert_eq!(body["threshold_n"], 3);
        assert_eq!(
            body["threshold_dids"]
                .as_array()
                .map(|array| array.len())
                .unwrap_or(0),
            3
        );
        assert!(body.get("single_did").is_none());
        assert!(body.get("open_set_members").is_none());
    }

    #[test]
    fn repair_request_body_serializes_with_strategy_tag() {
        let req = BottomRepairRequestBody {
            strategy: BottomRepairStrategy::HeadInWinner {
                head: BottomCandidateHead {
                    event_id: "ck:event:0196419b-0000-7000-8000-000000000000".into(),
                    issuer: Some("did:web:alice.example".into()),
                    hlc: None,
                    summary: None,
                },
            },
        };
        let encoded = serde_json::to_string(&req).unwrap();

        assert!(encoded.contains("\"strategy\":\"head_in_winner\""));
        assert!(encoded.contains("\"event_id\":\"ck:event:0196419b-0000-7000-8000-000000000000\""));
    }

    #[test]
    fn compaction_request_body_default_omits_max_control_moves() {
        let req = CompactionRequestBody {
            realm_id: "ck:realm:demo".into(),
            max_control_moves: None,
        };
        let encoded = serde_json::to_string(&req).unwrap();

        assert!(encoded.contains("\"realm_id\":\"ck:realm:demo\""));
        assert!(!encoded.contains("max_control_moves"));
    }
}
