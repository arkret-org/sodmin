//! Read-only HTTP client for the upstream starid did:webvh resolver.
//!
//! This client exists so the sodmin "Starid resolver status" panel
//! (round 35.4) can mirror the resolver's `/_arkret/root/identity/describe`
//! canonical `ServiceDescribe` response and its `x_starid_*` product
//! extensions.

use crate::api::client::{NO_BODY, api_client};
pub use crate::types::server::ServerDescribeDocument as StaridDescribe;
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
        "{}/_arkret/root/identity/describe",
        base.trim_end_matches('/')
    );
    let document: StaridDescribe = match api_client(&url, "GET", NO_BODY).await {
        Ok(document) => document,
        Err(error) => return Ok(Err(error)),
    };
    if let Err(error) = document.validate() {
        return Ok(Err(HttpError::message(format!(
            "invalid ServiceDescribe: {error}"
        ))));
    }
    Ok(Ok(document))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_uses_canonical_service_describe_and_starid_extensions() {
        let raw = r#"{
            "service_id": "ak:did_core:web:starid.example",
            "service_resolution": {
                "full_id": "did:web:starid.example",
                "method_history_head": "fixture-head",
                "version_id": "fixture-v1"
            },
            "trust_domain": "ak:trust_domain:starid.example",
            "service_kind": "identity_registry",
            "protocol_version": "1.0",
            "supported_profiles": ["ak.identity.webvh.v1"],
            "supported_operations": ["ak.root.identity.registry.read.describe"],
            "supported_bindings": [],
            "supported_features": [],
            "auth_metadata": {"mode": "production"},
            "limits": {},
            "rate_limit_policy": {"policy_version": "1", "entries": []},
            "plaintext_visibility": {},
            "implemented_features": ["identity.webvh.writer"],
            "claimed_profiles": [],
            "verified_profiles": [],
            "experimental_features": [],
            "compat_surfaces": [],
            "development_mode": false,
            "x_starid_registry_mode": "writer",
            "x_starid_supported_methods": ["did:webvh", "did:web"]
        }"#;
        let parsed: StaridDescribe = serde_json::from_str(raw).expect("parse");
        assert_eq!(parsed.service_id.as_str(), "ak:did_core:web:starid.example");
        parsed
            .validate()
            .expect("describe semantics should validate");
        assert_eq!(
            parsed.extra_str(&["x_starid_registry_mode"]).as_deref(),
            Some("writer")
        );
        assert_eq!(
            parsed.extra_string_list(&["x_starid_supported_methods"]),
            vec!["did:webvh", "did:web"]
        );
        assert_eq!(parsed.supported_profiles, vec!["ak.identity.webvh.v1"]);
    }
}
