//! Hand-defined contract DTOs for sodmin API clients.
//!
//! This module carries the soland/starid admin DTOs that have no
//! standalone upstream `*-admin-types` crate, named to match the OpenAPI
//! schema ids consumed by sodmin's typed clients. coauth DTOs are
//! imported directly from the `coauth_admin_types` crate at their call
//! sites (no facade re-export here).

pub mod starid {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    /// OpenAPI schema: `IdentityDescribeOutcome` from starid.
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct StaridDescribe {
        #[serde(default)]
        pub service_did: String,
        #[serde(default)]
        pub registry_mode: String,
        #[serde(default)]
        pub supported_methods: Vec<String>,
        #[serde(default)]
        pub supported_receipts: Vec<String>,
        #[serde(default)]
        pub protocol_version: String,
        #[serde(default)]
        pub profiles: Vec<String>,
        #[serde(default)]
        pub head_version_id: Option<String>,
        #[serde(default)]
        pub witness_count: u64,
        #[serde(default)]
        pub freshness: Option<DateTime<Utc>>,
        /// T8.3 — production hardening checklist snapshot.
        #[serde(default)]
        pub hardening: Option<crate::types::api::HardeningStatus>,
    }
}

#[cfg(test)]
mod tests {
    use super::starid::StaridDescribe;

    #[test]
    fn starid_contract_describe_tolerates_optional_status_fields() {
        let raw = r#"{
            "service_did": "did:web:starid.example",
            "registry_mode": "writer",
            "supported_methods": ["did:webvh"],
            "supported_receipts": ["starid-local-sha256-v1"],
            "protocol_version": "1.0",
            "profiles": []
        }"#;
        let describe: StaridDescribe = serde_json::from_str(raw).unwrap();
        assert_eq!(describe.service_did, "did:web:starid.example");
        assert_eq!(describe.witness_count, 0);
        assert!(describe.freshness.is_none());
    }
}
