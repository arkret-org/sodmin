//! Shared DTOs for the Notary / Seal / Control Move admin surface.
//!
//! The concrete wire shapes live in `soland-contracts` so the soland server and
//! sodmin admin UI use the same type definitions.

pub use soland_contracts::admin::seal::{
    AdminNotaryValue, BottomCandidateHead, BottomEntry, BottomKind, BottomKindExt,
    BottomRepairRequestBody, BottomRepairStrategy, CompactionOutcome, CompactionRequestBody,
    NotaryKind, NotaryReconfigRequestBody, SealDagSnapshot, SubmitControlMoveOutcome,
    bottom_kind_from_wire,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notary_value_summary_threshold() {
        let value = AdminNotaryValue {
            kind_raw: "threshold".to_owned(),
            threshold_k: Some(2),
            threshold_n: Some(3),
            threshold_dids: vec![
                "did:web:a.example".to_owned(),
                "did:web:b.example".to_owned(),
                "did:web:c.example".to_owned(),
            ],
            ..Default::default()
        };

        assert_eq!(value.kind(), Some(NotaryKind::Threshold));
        assert_eq!(value.summary(), "threshold(2/3)");
    }

    #[test]
    fn reconfigure_body_omits_unused_shape_fields() {
        let request = NotaryReconfigRequestBody {
            realm_id: "ak:realm:demo".to_owned(),
            kind: "single_did".to_owned(),
            single_did: Some("did:web:operator.example".to_owned()),
            ..Default::default()
        };
        let body = request.to_reconfigure_body();

        assert_eq!(body["kind"], "single_did");
        assert_eq!(body["single_did"], "did:web:operator.example");
        assert!(body.get("threshold_k").is_none());
        assert!(body.get("open_set_members").is_none());
    }

    #[test]
    fn seal_dag_deserializes_current_field_names() {
        let snapshot: SealDagSnapshot = serde_json::from_value(serde_json::json!({
            "realm_id": "ak:realm:demo",
            "covered_event_digests": ["ak:event:1"],
            "leaves": [{
                "seal_id": "ak:seal:1",
                "control_event_count": 1
            }]
        }))
        .expect("snapshot should deserialize");

        assert_eq!(
            snapshot.covered_event_digests,
            vec!["ak:event:1".to_owned()]
        );
        assert_eq!(snapshot.leaves[0].seal_id, "ak:seal:1");
    }

    #[test]
    fn compaction_outcome_uses_seal_id() {
        let response: CompactionOutcome = serde_json::from_value(serde_json::json!({
            "seal_id": "ak:seal:compact",
            "control_event_count": 3
        }))
        .expect("response should deserialize");

        assert_eq!(response.seal_id, "ak:seal:compact");
        assert_eq!(response.control_event_count, 3);
    }

    #[test]
    fn bottom_kind_parser_fails_closed() {
        assert!(matches!(
            bottom_kind_from_wire("conflict"),
            Some(BottomKind::Conflict)
        ));
        assert!(bottom_kind_from_wire("not_a_kind").is_none());
    }
}
