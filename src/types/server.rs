//! DTO shapes for the server status and describe admin surfaces.

use cokret_contracts::ops::HardeningStatus;
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ClaimedProfile {
    #[serde(default)]
    pub profile_id: String,
    #[serde(default)]
    pub claim_kind: Option<String>,
    #[serde(default)]
    pub claimed_at: Option<String>,
}

impl ClaimedProfile {
    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct VerifiedProfile {
    #[serde(default)]
    pub profile_id: String,
    #[serde(default)]
    pub claim_kind: Option<String>,
    #[serde(default)]
    pub cotest_run_id: Option<String>,
    #[serde(default)]
    pub artifact_digest: Option<String>,
    #[serde(default)]
    pub artifact_ref: Option<String>,
    #[serde(default)]
    pub cotest_issuer_did: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

impl VerifiedProfile {
    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    pub fn artifact_ref(&self) -> Option<&str> {
        self.artifact_ref.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CompatSurface {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

impl CompatSurface {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerDescribeResBody {
    #[serde(default)]
    pub service_did: String,
    /// Round 4 — required `trust_domain` per ServerDescribe v2 (spec
    /// a77b995). Anchors `ck.cross_signing.publish` proofs and federation
    /// canonical transcripts; mismatch is the wire-breaker.
    #[serde(default)]
    pub trust_domain: Option<String>,
    #[serde(default)]
    pub service_type: Option<String>,
    #[serde(default)]
    pub protocol_version: Option<String>,
    #[serde(default)]
    pub supported_profiles: Vec<String>,
    #[serde(default)]
    pub supported_features: Vec<String>,
    #[serde(default)]
    pub supported_operations: Vec<String>,
    #[serde(default)]
    pub supported_reducer_profiles: Vec<String>,
    #[serde(default)]
    pub supported_schema_profiles: Vec<String>,
    #[serde(default)]
    pub supported_bindings: Vec<serde_json::Value>,
    #[serde(default)]
    pub auth_metadata: Option<ServerDescribeAuthMetadata>,
    #[serde(default)]
    pub identity_registry_resolver: Option<IdentityRegistryResolverDescriptor>,
    #[serde(default)]
    pub admin_audience: Option<String>,
    #[serde(default)]
    pub openapi_version: Option<String>,
    #[serde(default)]
    pub schema_registry_version: Option<String>,
    #[serde(default)]
    pub event_kind_registry_version: Option<String>,
    #[serde(default)]
    pub registry: serde_json::Value,
    #[serde(default)]
    pub limits: serde_json::Value,
    /// Round 4 — `rate_limit` oneOf (window / token-bucket / disabled).
    /// Free-form JSON because the SDK still exposes it as `Value`.
    #[serde(default)]
    pub rate_limit: serde_json::Value,
    /// T1.4 — soland surfaces its dev-mode posture directly on
    /// `/_cokret/describe` (and `/health`). Sodmin uses this to
    /// render the red top-of-page banner. `None` for older servers that
    /// predate the field.
    #[serde(default)]
    pub development_mode: Option<bool>,
    /// T1.4 — `"development"` | `"production"`. Mirrors
    /// [`Self::development_mode`]; when `development_mode == true`,
    /// soland accepts unsigned / weakly-signed envelopes.
    #[serde(default)]
    pub proof_verifier_mode: Option<String>,
    /// T1.4 — effective admin-API auth posture:
    /// `"development"` | `"did_allowlist"` | `"oauth_introspection"` | `"closed"`.
    #[serde(default)]
    pub admin_auth_mode: Option<String>,
    /// Round 4 — concrete `implemented_features` list (subset of
    /// supported_features that this build actually wires up). Distinct
    /// from `supported_features` which advertises the capability surface.
    #[serde(default)]
    pub implemented_features: Vec<String>,
    /// T6.1 — profiles the server has been independently verified to
    /// implement against the spec test suite. Rendered as green
    /// "verified" chips.
    #[serde(default)]
    pub verified_profiles: Vec<VerifiedProfile>,
    /// T6.1 — profiles the operator self-claims support for. Rendered
    /// as yellow "self-claimed" chips because they lack third-party
    /// verification.
    #[serde(default)]
    pub claimed_profiles: Vec<ClaimedProfile>,
    /// T6.1 — experimental features the server exposes. Rendered as
    /// blue chips with an "unstable" warning.
    #[serde(default)]
    pub experimental_features: Vec<String>,
    /// T6.1 — compatibility surfaces (legacy / shim endpoints). Grey
    /// chips.
    #[serde(default)]
    pub compat_surfaces: Vec<CompatSurface>,
    /// Round 4 — `plaintext_visibility` snapshot (services whose plaintext
    /// bodies remain readable on this deployment).
    #[serde(default)]
    pub plaintext_visibility: Vec<String>,
    /// T8.3 — production hardening checklist snapshot, mirrored from
    /// `/health`. Older servers that predate the field omit it.
    #[serde(default)]
    pub hardening: Option<HardeningStatus>,
}

impl ServerDescribeResBody {
    /// Round 4 — render-time check used by the ServerDescribe v2 admin
    /// view. When `development_mode == true` AND `verified_profiles` is
    /// non-empty the server is making contradictory claims (relaxed
    /// proof verifier breaks the verification chain). The UI surfaces a
    /// red warning banner in this case.
    pub fn dev_mode_with_verified_profiles(&self) -> bool {
        self.development_mode.unwrap_or(false) && !self.verified_profiles.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerDescribeAuthMetadata {
    #[serde(default)]
    pub issuer_did: Option<String>,
    #[serde(default)]
    pub oauth_issuer: Option<String>,
    #[serde(default)]
    pub openid_configuration: Option<String>,
    #[serde(default)]
    pub required_audience: Option<String>,
    #[serde(default)]
    pub admin_audience: Option<String>,
    #[serde(default)]
    pub supported_auth_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityRegistryResolverDescriptor {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub delegated_resolver: Option<IdentityRegistryDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityRegistryDescriptor {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub resolver: Option<String>,
    #[serde(default)]
    pub proof_required_for_pairwise: Option<bool>,
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
pub struct ServerStatusResponse {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub results: Vec<ServerStatusComponent>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ServerDescribeResBody;

    #[test]
    fn server_describe_accepts_principal_server_profile_status() {
        let describe: ServerDescribeResBody = serde_json::from_value(json!({
            "service_did": "did:web:soland.local",
            "service_type": "principal_server",
            "protocol_version": "1.0",
            "supported_profiles": ["ck.profile.principal_server.v1"],
            "supported_features": ["events.describe", "events.submit"],
            "supported_operations": ["ck.self.events.submit"],
            "supported_reducer_profiles": ["ck.reducer.v1"],
            "supported_schema_profiles": ["ck.schema.core.v1"],
            "limits": {
                "profile_status": {
                    "conformance": "limited_reference",
                    "implemented_surfaces": ["principal_server", "events_api_minimal"]
                }
            }
        }))
        .expect("soland server describe should deserialize");

        assert_eq!(describe.service_did, "did:web:soland.local");
        assert_eq!(describe.service_type.as_deref(), Some("principal_server"));
        assert_eq!(describe.protocol_version.as_deref(), Some("1.0"));
        assert_eq!(
            describe.supported_reducer_profiles,
            vec!["ck.reducer.v1".to_string()]
        );
        assert_eq!(
            describe.limits["profile_status"]["conformance"],
            "limited_reference"
        );
    }

    #[test]
    fn server_describe_accepts_coauth_issuer_and_registry() {
        let describe: ServerDescribeResBody = serde_json::from_value(json!({
            "service_did": "did:web:auth.example.com",
            "service_type": "auth_account_server",
            "protocol_version": "1.0",
            "supported_profiles": ["ck.profile.auth_account.v1"],
            "auth_metadata": {
                "issuer_did": "did:web:auth.example.com#issuer",
                "oauth_issuer": "https://auth.example.com"
            },
            "identity_registry_resolver": {
                "mode": "delegated_resolver",
                "endpoint": "https://auth.example.com/_cokret/root/identity/resolve",
                "delegated_resolver": {
                    "kind": "public_did_resolver",
                    "resolver": "https://resolver.example.com/"
                }
            }
        }))
        .expect("coauth server describe should deserialize");

        assert_eq!(
            describe
                .auth_metadata
                .as_ref()
                .and_then(|metadata| metadata.issuer_did.as_deref()),
            Some("did:web:auth.example.com#issuer")
        );
        assert_eq!(
            describe
                .identity_registry_resolver
                .as_ref()
                .and_then(|registry| registry.delegated_resolver.as_ref())
                .and_then(|delegated| delegated.resolver.as_deref()),
            Some("https://resolver.example.com/")
        );
    }

    #[test]
    fn server_describe_reads_supported_profiles() {
        let describe: ServerDescribeResBody = serde_json::from_value(json!({
            "service_did": "did:web:identity.example",
            "supported_profiles": ["ck.profile.identity_registry.v1"]
        }))
        .expect("supported_profiles should deserialize");

        assert_eq!(
            describe.supported_profiles,
            vec!["ck.profile.identity_registry.v1".to_string()]
        );
    }

    #[test]
    fn server_describe_reads_conformance_buckets() {
        let describe: ServerDescribeResBody = serde_json::from_value(json!({
            "service_did": "did:web:soland.local",
            "verified_profiles": [{
                "profile_id": "ck.profile.principal_server.v1",
                "claim_kind": "cotest_verified",
                "artifact_ref": "ck:artifact:principal-server-run"
            }],
            "claimed_profiles": [{
                "profile_id": "ck.profile.identity_registry.v1",
                "claim_kind": "self_claimed",
                "claimed_at": "2026-06-05T00:00:00Z"
            }],
            "experimental_features": ["events.replay.v2"],
            "compat_surfaces": [{
                "name": "federation.bridge",
                "kind": "shim"
            }],
            "plaintext_visibility": ["floria"],
        }))
        .expect("conformance buckets should deserialize");

        assert_eq!(
            describe.verified_profiles[0].profile_id(),
            "ck.profile.principal_server.v1"
        );
        assert_eq!(
            describe.verified_profiles[0].artifact_ref(),
            Some("ck:artifact:principal-server-run")
        );
        assert_eq!(
            describe.claimed_profiles[0].profile_id(),
            "ck.profile.identity_registry.v1"
        );
        assert_eq!(
            describe.experimental_features,
            vec!["events.replay.v2".to_string()]
        );
        assert_eq!(describe.compat_surfaces[0].name(), "federation.bridge");
        assert_eq!(describe.plaintext_visibility, vec!["floria".to_string()]);
    }
}
