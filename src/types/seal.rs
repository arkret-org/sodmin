//! Shared DTOs for the Notary / Seal / Control Move admin surface.
//!
//! The concrete wire shapes live in `soland-contracts` so the soland server and
//! sodmin admin UI use the same type definitions.

pub use soland_contracts::admin::seal::{
    AdminNotaryValue, BottomEntry, BottomKindExt, SealChainSnapshot, bottom_kind_from_wire,
};

#[cfg(test)]
mod tests {
    use arkret_wire::BottomKind;

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
        })
    }

    #[test]
    fn notary_value_uses_one_authority() {
        let value: AdminNotaryValue = serde_json::from_value(serde_json::json!({
            "notary": {"signer": notary_signer("a"), "max_clock_error_ms": 1000}
        }))
        .unwrap();
        assert_eq!(
            value.notary.signer.verification_method.as_str(),
            "did:web:a.example#notary-1"
        );
        let mut retired = serde_json::to_value(value).unwrap();
        retired["notary"]["kind"] = serde_json::json!("quorum");
        assert!(serde_json::from_value::<AdminNotaryValue>(retired).is_err());
    }

    #[test]
    fn seal_chain_deserializes_current_field_names() {
        let snapshot: SealChainSnapshot = serde_json::from_value(serde_json::json!({
            "realm_id": "ak:realm:AdMiEvn36jc6t91sjSYuDEZkZX0Ci3UXAX2YuCRDT2GQ",
            "covered_event_digests": ["ak:event:AVxWLctdfSsE1Zydw00Ka5bXOeyQ9ocw6ZlO7eAhe5i3"],
            "head": {
                "seal_id": "ak:seal:sha256:c53617efdd06a540dfc23c88a059db52bcb4a8c94fee2428a05ed80674230056",
                "signer": "did:web:a.example#notary-1",
                "control_event_count": 1
            }
        }))
        .expect("snapshot should deserialize");

        assert_eq!(
            snapshot.covered_event_digests,
            vec!["ak:event:AVxWLctdfSsE1Zydw00Ka5bXOeyQ9ocw6ZlO7eAhe5i3".to_owned()]
        );
        assert_eq!(
            snapshot.head.expect("confirmed head").seal_id,
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
