//! DTO shapes for the server status and describe admin surfaces.
//!
//! The `/_cokret/describe` payload is the SDK-authoritative
//! [`ServerDescription`] (`cokret_core::models::api`). sodmin does not
//! mirror it; service-proprietary top-level extensions (coauth's
//! `identity_registry_resolver`, `admin_audience`, …) are captured in a
//! flattened `extra` envelope and read by the view layer on demand.

// (The `ClaimedProfileEntry` / `VerifiedProfileEntry` /
// `CompatSurfaceEntry` element types are reachable through the SDK
// directly; sodmin views consume them via the `ServerDescription`
// fields and need no local re-export.)
pub use cokret_core::models::ServerDescription;
use serde::{Deserialize, Serialize};

// ── Server info types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerInfo {
    #[serde(default)]
    pub server_version: String,
    #[serde(default)]
    pub protocol_version: Option<String>,
    #[serde(default)]
    pub server_name: Option<String>,
    #[serde(default)]
    pub uptime: Option<u64>,
}

/// `/_cokret/describe` response envelope.
///
/// The protocol-authoritative fields deserialize into the SDK
/// [`ServerDescription`] (strict: `service_did: Did`,
/// `trust_domain: TypedTrustDomainId`, `development_mode: bool`, …).
/// Any additional top-level keys a service emits beyond the spec shape
/// (e.g. coauth's `identity_registry_resolver` extension block) land in
/// [`Self::extra`]; views read them via [`Self::extra_str`] so the wire
/// type itself never grows hand-written mirrors of upstream extensions.
/// (`Serialize` is only needed for the localStorage TTL cache used by
/// the dashboard — `extra` always deserializes to a JSON object, so the
/// flattened round-trip is well-formed.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDescribeDocument {
    #[serde(flatten)]
    pub description: ServerDescription,
    /// Top-level keys not consumed by [`ServerDescription`].
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl std::ops::Deref for ServerDescribeDocument {
    type Target = ServerDescription;

    fn deref(&self) -> &ServerDescription {
        &self.description
    }
}

impl ServerDescribeDocument {
    /// Walk `path` inside the extension envelope and return the string
    /// leaf, if any.
    pub fn extra_str(&self, path: &[&str]) -> Option<String> {
        let mut current = &self.extra;
        for key in path {
            current = current.get(*key)?;
        }
        current.as_str().map(ToOwned::to_owned)
    }

    /// Round 4 — render-time check used by the ServerDescribe v2 admin
    /// view. When `development_mode == true` AND `verified_profiles` is
    /// non-empty the server is making contradictory claims (relaxed
    /// proof verifier breaks the verification chain). The UI surfaces a
    /// red warning banner in this case.
    pub fn dev_mode_with_verified_profiles(&self) -> bool {
        self.description.development_mode && !self.description.verified_profiles.is_empty()
    }

    /// Render the canonical `plaintext_visibility` advertisement as a list
    /// of the declared plaintext data-class names for display. An empty
    /// list means the service claims no plaintext / reversible-derived
    /// classes (the E2EE default).
    pub fn plaintext_visibility_entries(&self) -> Vec<String> {
        self.description
            .plaintext_visibility
            .data_classes
            .iter()
            .filter_map(|class| {
                serde_json::to_value(class)
                    .ok()
                    .and_then(|v| v.as_str().map(ToOwned::to_owned))
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerStats {
    #[serde(default)]
    pub actor_count: u64,
    #[serde(default)]
    pub active_actor_count: u64,
    #[serde(default)]
    pub realm_count: u64,
    #[serde(default)]
    pub report_count: u64,
    #[serde(default)]
    pub federation_peer_count: u64,
    #[serde(default)]
    pub applet_count: u64,
    #[serde(default)]
    pub agent_count: u64,
    #[serde(default)]
    pub blob_count: u64,
    #[serde(default)]
    pub blob_total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerStatusComponent {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerStatusOutcome {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub results: Vec<ServerStatusComponent>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ServerDescribeDocument;

    /// Minimal payload satisfying every REQUIRED `ServerDescription`
    /// field (Round 4 ServiceDescribe v2 shape, as soland emits it).
    fn base_describe() -> serde_json::Value {
        json!({
            "service_did": "did:web:soland.local",
            "trust_domain": "ck:trust_domain:soland.local",
            "service_type": "principal_server",
            "protocol_version": "1.0",
            "supported_profiles": ["ck.profile.principal_server.v1"],
            "supported_operations": ["ck.self.events.command.submit"],
            "supported_bindings": [],
            "supported_features": ["events.describe", "events.submit"],
            "auth_metadata": { "mode": "production" },
            "limits": {
                "profile_status": {
                    "conformance": "limited_reference",
                    "implemented_surfaces": ["principal_server", "events_api_minimal"]
                }
            },
            "plaintext_visibility": { "max_visibility": "private_plaintext", "data_classes": ["message_content"] },
            "implemented_features": ["events.describe"],
            "claimed_profiles": [{
                "profile_id": "ck.profile.principal_server.v1",
                "claim_kind": "self_claimed"
            }],
            "verified_profiles": [],
            "experimental_features": ["events.replay.v2"],
            "compat_surfaces": [{
                "name": "federation.bridge",
                "kind": "external_interop"
            }],
            "development_mode": false,
        })
    }

    #[test]
    fn describe_document_parses_sdk_authoritative_shape() {
        let describe: ServerDescribeDocument =
            serde_json::from_value(base_describe()).expect("soland describe should deserialize");

        assert_eq!(describe.service_did.as_str(), "did:web:soland.local");
        assert_eq!(
            describe.trust_domain.as_str(),
            "ck:trust_domain:soland.local"
        );
        assert_eq!(describe.service_type, "principal_server");
        assert!(!describe.development_mode);
        assert_eq!(
            describe.claimed_profiles[0].profile_id,
            "ck.profile.principal_server.v1"
        );
        assert_eq!(describe.compat_surfaces[0].name, "federation.bridge");
        assert_eq!(
            describe.limits["profile_status"]["conformance"],
            "limited_reference"
        );
        assert_eq!(
            describe.plaintext_visibility_entries(),
            vec!["message_content".to_string()]
        );
        assert!(!describe.dev_mode_with_verified_profiles());
    }

    #[test]
    fn describe_document_captures_service_extensions_in_extra() {
        let mut payload = base_describe();
        let object = payload.as_object_mut().unwrap();
        // coauth-proprietary top-level extension fields.
        object.insert(
            "identity_registry_resolver".to_owned(),
            json!({
                "mode": "delegated_resolver",
                "endpoint": "https://auth.example.com/api/v1/identity/resolve",
                "delegated_resolver": {
                    "kind": "public_did_resolver",
                    "resolver": "https://resolver.example.com/"
                }
            }),
        );
        object.insert("admin_audience".to_owned(), json!("urn:coauth:admin"));

        let describe: ServerDescribeDocument =
            serde_json::from_value(payload).expect("extended describe should deserialize");

        assert_eq!(
            describe.extra_str(&[
                "identity_registry_resolver",
                "delegated_resolver",
                "resolver"
            ]),
            Some("https://resolver.example.com/".to_string())
        );
        assert_eq!(
            describe.extra_str(&["admin_audience"]),
            Some("urn:coauth:admin".to_string())
        );
        // Spec fields must NOT leak into the extension envelope.
        assert!(describe.extra.get("service_did").is_none());
    }
}
