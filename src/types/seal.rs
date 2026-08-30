//! Shared DTOs for the Notary / Seal / Control Move admin surface.
//!
//! The concrete wire shapes live in `soland-contracts` so the soland server and
//! sodmin admin UI use the same type definitions.

pub use soland_contracts::admin::seal::{
    AdminNotaryValue, BottomCandidateHead, BottomEntry, BottomKind, BottomKindExt, SealDagSnapshot,
    bottom_kind_from_wire,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn notary_signer(actor: &str) -> serde_json::Value {
        serde_json::json!({
            "actor_id": {
                "kind": "service",
                "service_id": format!("ak:did_core:web:{actor}.example"),
            },
            "verification_method": format!("did:web:{actor}.example#notary-1"),
            "key_kind": "ed25519_raw32",
            "jose_algorithm": "Ed25519",
            "frozen_public_key_b64u": "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8",
            "frozen_public_key_digest": "sha256:630dcd2966c4336691125448bbb25b4ff412a49c732db2c8abc1b8581bd710dd",
        })
    }

    #[test]
    fn notary_value_summary_threshold() {
        let value: AdminNotaryValue = serde_json::from_value(serde_json::json!({
            "notary": {
                "kind": "threshold",
                "signers": [
                    notary_signer("a"),
                    notary_signer("b"),
                    notary_signer("c"),
                ],
                "threshold": 2,
                "forensic_attribution": "quorum_intersection"
            }
        }))
        .expect("threshold notary should deserialize");

        assert_eq!(value.kind_label(), "threshold");
        assert_eq!(value.summary(), "threshold(2/3)");
    }

    #[test]
    fn seal_dag_deserializes_current_field_names() {
        let snapshot: SealDagSnapshot = serde_json::from_value(serde_json::json!({
            "realm_id": "ak:realm:AdMiEvn36jc6t91sjSYuDEZkZX0Ci3UXAX2YuCRDT2GQ",
            "covered_event_digests": ["ak:event:AVxWLctdfSsE1Zydw00Ka5bXOeyQ9ocw6ZlO7eAhe5i3"],
            "leaves": [{
                "seal_id": "ak:seal:sha256:c53617efdd06a540dfc23c88a059db52bcb4a8c94fee2428a05ed80674230056",
                "control_event_count": 1
            }]
        }))
        .expect("snapshot should deserialize");

        assert_eq!(
            snapshot.covered_event_digests,
            vec!["ak:event:AVxWLctdfSsE1Zydw00Ka5bXOeyQ9ocw6ZlO7eAhe5i3".to_owned()]
        );
        assert_eq!(
            snapshot.leaves[0].seal_id,
            "ak:seal:sha256:c53617efdd06a540dfc23c88a059db52bcb4a8c94fee2428a05ed80674230056"
        );
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
