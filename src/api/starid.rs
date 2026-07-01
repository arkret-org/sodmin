//! Read-only HTTP client for the upstream starid did:webvh resolver.
//!
//! This client exists so the sodmin "Starid resolver status" panel
//! (round 35.4) can mirror the resolver's `/_cokret/root/identity/describe`
//! response — service DID + protocol version, head version_id, witness
//! count, and freshness.

use crate::api::client::{api_client, NO_BODY};
pub use crate::api::contracts::starid::StaridDescribe;
use crate::utils::net::error::HttpError;
use crate::utils::net::session;

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
    let url = format!(
        "{}/_cokret/root/identity/describe",
        base.trim_end_matches('/')
    );
    Ok(api_client(&url, "GET", NO_BODY).await)
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
            "profiles": ["ck.identity.webvh.v1", "ck.identity.registry.v1"],
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
