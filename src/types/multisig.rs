//! DTO adapters for the multi-sig partial-signature admin surface.
//!
//! Pending rows come from `soland-core`. The partial-submit request/response
//! stays local because the current UI posts an admin-scoped note while the
//! soland server's lower-level endpoint accepts raw partial signature material.

use serde::{Deserialize, Serialize};
pub use soland_core::admin::seal::{
    MultisigPendingEntry as PendingMultisigSeal, MultisigPendingOutcome,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitPartialSignatureRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubmitPartialSignatureResponse {
    #[serde(default)]
    pub seal_id: String,
    #[serde(default)]
    pub collected_partials: u32,
    #[serde(default)]
    pub threshold_k: u32,
    #[serde(default)]
    pub threshold_met: bool,
    #[serde(default)]
    pub status: String,
}

impl SubmitPartialSignatureResponse {
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
            seal_id: "ck:seal:1".to_owned(),
            threshold_k: 3,
            threshold_n: 5,
            collected_partials: 1,
            ..Default::default()
        };

        assert_eq!(row.seal_id, "ck:seal:1");
        assert_eq!(row.remaining(), 2);
        assert_eq!(row.threshold_label(), "3 of 5");
        assert!(!row.is_threshold_met());
    }

    #[test]
    fn partial_submit_response_accepts_server_field_names() {
        let response: SubmitPartialSignatureResponse = serde_json::from_value(serde_json::json!({
            "seal_id": "ck:seal:1",
            "collected_partials": 2,
            "threshold_k": 2,
            "status": "aggregated"
        }))
        .expect("response should deserialize");

        assert_eq!(response.seal_id, "ck:seal:1");
        assert_eq!(response.collected_partials, 2);
        assert_eq!(response.threshold_k, 2);
        assert!(response.threshold_met());
    }

    #[test]
    fn submit_partial_request_omits_empty_note() {
        let request = SubmitPartialSignatureRequest { note: None };
        let encoded = serde_json::to_string(&request).unwrap();

        assert_eq!(encoded, "{}");
    }
}
