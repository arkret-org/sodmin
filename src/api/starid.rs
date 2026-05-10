//! Read-only HTTP client for the upstream starid did:webvh resolver.
//!
//! This client exists so the sodmin "Starid resolver status" panel
//! (round 35.4) can mirror the resolver's `/api/v1/identity/describe`
//! response — service DID + protocol version, head version_id, witness
//! count, and freshness — without having to graduate starid into the
//! shared `coauth-admin-types` crate yet.
//!
//! The wire shape matches starid's `wire::IdentityDescribeResponse`
//! exactly. Optional fields are tolerated as missing so older starid
//! deployments (pre-C35.4) that haven't started emitting the aggregate
//! resolver-status fields still render as a healthy panel with the
//! aggregate cells stamped "-".

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::api::client::api_client;
use crate::utils::error::HttpError;
use crate::utils::session;

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
}

/// Raised when the operator has not configured `starid_public_url`. The
/// page surfaces this as a "Starid not configured" empty-state instead
/// of an error banner so the operator knows to set the deployment
/// config rather than chase a network failure.
pub struct StaridNotConfigured;

/// Fetch the upstream starid resolver describe envelope. Returns
/// `Err(StaridNotConfigured)` when no public URL is wired.
pub async fn get_describe() -> Result<Result<StaridDescribe, HttpError>, StaridNotConfigured> {
    let Some(base) = session::starid_public_url() else {
        return Err(StaridNotConfigured);
    };
    let url = format!("{}/api/v1/identity/describe", base.trim_end_matches('/'));
    Ok(api_client(&url, "GET", None).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_round_trips_full_payload() {
        let raw = r#"{
            "service_did": "did:web:starid.example",
            "registry_mode": "writer",
            "supported_methods": ["did:webvh", "did:web"],
            "supported_receipts": ["starid-local-sha256-v1"],
            "protocol_version": "1.0",
            "profiles": ["cx.identity.webvh.v1", "cx.identity.registry.v1"],
            "head_version_id": "42-zABCDEF",
            "witness_count": 7,
            "freshness": "2026-05-09T10:11:12Z"
        }"#;
        let parsed: StaridDescribe = serde_json::from_str(raw).expect("parse");
        assert_eq!(parsed.service_did, "did:web:starid.example");
        assert_eq!(parsed.registry_mode, "writer");
        assert_eq!(parsed.supported_methods, vec!["did:webvh", "did:web"]);
        assert_eq!(parsed.head_version_id.as_deref(), Some("42-zABCDEF"));
        assert_eq!(parsed.witness_count, 7);
        assert!(parsed.freshness.is_some());
        assert_eq!(parsed.profiles.len(), 2);
    }

    #[test]
    fn describe_tolerates_pre_c35_4_payload() {
        // Starid before round 35.4 only emitted the bridge-discovery
        // fields. The aggregate cells must default to empty / zero so
        // the panel still renders a healthy card instead of failing
        // the JSON parse.
        let raw = r#"{
            "service_did": "did:web:starid.example",
            "registry_mode": "writer",
            "supported_methods": ["did:webvh"],
            "supported_receipts": ["starid-local-sha256-v1"],
            "protocol_version": "1.0",
            "profiles": []
        }"#;
        let parsed: StaridDescribe = serde_json::from_str(raw).expect("parse");
        assert_eq!(parsed.head_version_id, None);
        assert_eq!(parsed.witness_count, 0);
        assert!(parsed.freshness.is_none());
    }
}
