//! DTO shapes for the server status and describe admin surfaces.
//!
//! The `/_arkret/describe` payload is the SDK-authoritative
//! [`ServiceDescribe`] (`arkret_models_discovery`). sodmin does not
//! mirror it; service-proprietary top-level extensions are read from the
//! SDK model's flattened `extensions` map.

// (The `ClaimedProfileEntry` / `VerifiedProfileEntry` /
// `CompatSurfaceEntry` element types are reachable through the SDK
// directly; sodmin views consume them via the `ServiceDescribe`
// fields and need no local re-export.)
pub use arkret_models_discovery::ServiceDescribe;
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

/// `/_arkret/describe` response envelope.
///
/// The protocol-authoritative fields deserialize into the SDK
/// [`ServiceDescribe`] (strict: `service_id: ServiceId`, an exact
/// `service_resolution` commitment, `trust_domain: TypedTrustDomainId`,
/// `development_mode: bool`, …).
/// Any additional top-level keys a service emits beyond the spec shape
/// land in [`ServiceDescribe::extensions`]; views read them via
/// [`Self::extra_str`] so the wire type itself never grows hand-written
/// mirrors of upstream extensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ServerDescribeDocument {
    pub description: ServiceDescribe,
}

impl std::ops::Deref for ServerDescribeDocument {
    type Target = ServiceDescribe;

    fn deref(&self) -> &ServiceDescribe {
        &self.description
    }
}

impl ServerDescribeDocument {
    /// Enforce the SDK's semantic checks, including full-id-to-core-id
    /// consistency for the published service resolution commitment.
    pub fn validate(&self) -> arkret_wire::Result<()> {
        self.description.validate()
    }

    /// Walk `path` inside the extension envelope.
    pub fn extra_value<'a>(&'a self, path: &[&str]) -> Option<&'a serde_json::Value> {
        let mut current = path
            .first()
            .and_then(|key| self.description.extensions.get(*key))?;
        for key in &path[1..] {
            current = current.get(*key)?;
        }
        Some(current)
    }

    /// Walk `path` inside the extension envelope and return the string
    /// leaf, if any.
    pub fn extra_str(&self, path: &[&str]) -> Option<String> {
        self.extra_value(path)?.as_str().map(ToOwned::to_owned)
    }

    /// Return a string-array extension, dropping non-string elements.
    pub fn extra_string_list(&self, path: &[&str]) -> Vec<String> {
        self.extra_value(path)
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// When `development_mode == true` and `verified_profiles` is
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

    /// Minimal payload satisfying every required `ServiceDescribe` field.
    fn base_describe() -> serde_json::Value {
        json!({
            "service_id": "ak:did_core:web:soland.local",
            "service_resolution": {
                "full_id": "did:web:soland.local",
                "method_history_head": "fixture-head",
                "version_id": "fixture-v1"
            },
            "trust_domain": "ak:trust_domain:soland.local",
            "service_kind": "principal_server",
            "protocol_version": "1.0",
            "supported_profiles": ["ak.profile.principal_server.v1"],
            "supported_operations": ["ak.self.events.command.submit"],
            "supported_bindings": [],
            "supported_features": ["events.describe", "events.submit"],
            "auth_metadata": { "mode": "production" },
            "limits": {
                "profile_status": {
                    "conformance": "limited_reference",
                    "implemented_surfaces": ["principal_server", "events_api_minimal"]
                }
            },
            "rate_limit_policy": { "policy_version": "1", "entries": [] },
            "plaintext_visibility": { "max_visibility": "private_plaintext", "data_classes": ["message_content"] },
            "implemented_features": ["events.describe"],
            "claimed_profiles": [{
                "profile_id": "ak.profile.principal_server.v1",
                "claim_kind": "self_claimed"
            }],
            "verified_profiles": [],
            "experimental_features": [],
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

        assert_eq!(describe.service_id.as_str(), "ak:did_core:web:soland.local");
        describe
            .validate()
            .expect("describe semantics should validate");
        assert_eq!(
            describe.trust_domain.as_str(),
            "ak:trust_domain:soland.local"
        );
        assert_eq!(
            describe.service_kind,
            arkret_wire::ServiceKind::PrincipalServer
        );
        assert!(!describe.development_mode);
        assert_eq!(
            describe.claimed_profiles[0].profile_id,
            "ak.profile.principal_server.v1"
        );
        assert_eq!(describe.compat_surfaces[0].name, "federation.bridge");
        assert_eq!(
            describe.limits.extensions["profile_status"]["conformance"],
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
        assert!(!describe.description.extensions.contains_key("service_id"));
    }
}
