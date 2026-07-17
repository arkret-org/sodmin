//! DTO adapters for the multi-sig partial-signature admin surface.
//!
//! Pending rows come from `soland-contracts`. The partial-submit request/outcome
//! stays local because soland accepts raw partial signature material.

use serde::{Deserialize, Serialize};
pub use soland_contracts::admin::seal::{
    MultisigPendingEntry as PendingMultisigSeal, MultisigPendingOutcome,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitPartialSignatureRequest {
    pub signer_did: String,
    pub signature_b64: String,
    pub kid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitPartialSignatureOutcome {
    #[serde(default)]
    pub seal_id: String,
    #[serde(default)]
    #[serde(rename = "collected")]
    pub collected_partials: u32,
    #[serde(default)]
    #[serde(rename = "threshold")]
    pub threshold_k: u32,
    #[serde(default)]
    pub threshold_met: bool,
    #[serde(default)]
    pub status: String,
}

impl SubmitPartialSignatureOutcome {
    pub fn threshold_met(&self) -> bool {
        self.threshold_met || self.status == "aggregated"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_row_helpers_come_from_shared_type() {
        let row = PendingMultisigSeal {
            seal_id: "ak:seal:1".to_owned(),
            threshold_k: 3,
            threshold_n: 5,
            collected_partials: 1,
            ..Default::default()
        };

        assert_eq!(row.seal_id, "ak:seal:1");
        assert_eq!(row.remaining(), 2);
        assert_eq!(row.threshold_label(), "3 of 5");
        assert!(!row.is_threshold_met());
    }

    #[test]
    fn partial_submit_outcome_accepts_server_field_names() {
        let response: SubmitPartialSignatureOutcome = serde_json::from_value(serde_json::json!({
            "seal_id": "ak:seal:1",
            "collected": 2,
            "threshold": 2,
            "status": "aggregated"
        }))
        .expect("response should deserialize");

        assert_eq!(response.seal_id, "ak:seal:1");
        assert_eq!(response.collected_partials, 2);
        assert_eq!(response.threshold_k, 2);
        assert!(response.threshold_met());
    }

    #[test]
    fn submit_partial_request_matches_server_field_names() {
        let request = SubmitPartialSignatureRequest {
            signer_did: "did:web:admin.example".to_owned(),
            signature_b64: "abc".to_owned(),
            kid: "did:web:admin.example#key-1".to_owned(),
        };
        let encoded = serde_json::to_string(&request).unwrap();

        assert_eq!(
            encoded,
            r#"{"signer_did":"did:web:admin.example","signature_b64":"abc","kid":"did:web:admin.example#key-1"}"#
        );
    }
}
