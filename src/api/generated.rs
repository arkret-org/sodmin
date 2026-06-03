//! Generated/shared contract facade for sodmin API clients.
//!
//! This module is the single import point for DTOs that are owned by
//! upstream contracts.  Where an upstream shared crate exists
//! (`coauth-admin-types`) we re-export it directly.  For
//! soland admin surfaces that do not yet have a standalone
//! `soland-admin-types` crate, the DTOs live here with names that match
//! the OpenAPI schema ids consumed by sodmin's typed clients.

pub mod coauth_admin {
    #[allow(unused_imports)]
    pub use coauth_admin_types::*;
}

pub mod soland_admin {
    use serde::{Deserialize, Serialize};

    /// Status of a single capability grant. Mirrors soland's authz
    /// reducer state machine.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum AuthzGrantStatus {
        Active,
        Revoked,
        Expired,
    }

    impl AuthzGrantStatus {
        pub fn label(&self) -> &'static str {
            match self {
                AuthzGrantStatus::Active => "Active",
                AuthzGrantStatus::Revoked => "Revoked",
                AuthzGrantStatus::Expired => "Expired",
            }
        }

        pub fn from_wire(s: &str) -> Option<Self> {
            match s {
                "active" => Some(AuthzGrantStatus::Active),
                "revoked" => Some(AuthzGrantStatus::Revoked),
                "expired" => Some(AuthzGrantStatus::Expired),
                _ => None,
            }
        }
    }

    /// OpenAPI schema: `AuthzCapabilityGrant`.
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct AuthzCapabilityGrant {
        #[serde(default)]
        pub grant_id: String,
        #[serde(default)]
        pub holder_did: String,
        #[serde(default)]
        pub peer_did: String,
        #[serde(default)]
        pub scope: String,
        #[serde(default)]
        pub status: String,
        #[serde(default)]
        pub granted_at: Option<String>,
        #[serde(default)]
        pub expires_at: Option<String>,
        #[serde(default)]
        pub note: Option<String>,
    }

    impl AuthzCapabilityGrant {
        pub fn status_typed(&self) -> AuthzGrantStatus {
            AuthzGrantStatus::from_wire(&self.status).unwrap_or(AuthzGrantStatus::Active)
        }

        pub fn is_revocable(&self) -> bool {
            matches!(self.status_typed(), AuthzGrantStatus::Active) && !self.grant_id.is_empty()
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct AuthzGrantFilter {
        pub holder: String,
        pub peer: String,
        pub scope: String,
    }

    impl AuthzGrantFilter {
        pub fn is_empty(&self) -> bool {
            self.holder.trim().is_empty()
                && self.peer.trim().is_empty()
                && self.scope.trim().is_empty()
        }
    }

    pub fn filter_grants(
        grants: &[AuthzCapabilityGrant],
        filter: &AuthzGrantFilter,
    ) -> Vec<AuthzCapabilityGrant> {
        let holder = filter.holder.trim().to_lowercase();
        let peer = filter.peer.trim().to_lowercase();
        let scope = filter.scope.trim().to_lowercase();
        grants
            .iter()
            .filter(|g| holder.is_empty() || g.holder_did.to_lowercase().contains(&holder))
            .filter(|g| peer.is_empty() || g.peer_did.to_lowercase().contains(&peer))
            .filter(|g| scope.is_empty() || g.scope.to_lowercase().contains(&scope))
            .cloned()
            .collect()
    }

    /// Lifecycle state of a moderation report.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum ReportStatus {
        Open,
        Resolved,
        Dismissed,
    }

    impl ReportStatus {
        pub fn label(&self) -> &'static str {
            match self {
                ReportStatus::Open => "Open",
                ReportStatus::Resolved => "Resolved",
                ReportStatus::Dismissed => "Dismissed",
            }
        }

        pub fn from_wire(s: &str) -> Option<Self> {
            match s {
                "open" => Some(ReportStatus::Open),
                "resolved" => Some(ReportStatus::Resolved),
                "dismissed" => Some(ReportStatus::Dismissed),
                _ => None,
            }
        }
    }

    /// OpenAPI schema: `ModerationReport`.
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct ModerationReport {
        #[serde(default)]
        pub report_id: String,
        #[serde(default)]
        pub reporter_did: String,
        #[serde(default)]
        pub target_did: Option<String>,
        #[serde(default)]
        pub space_id: Option<String>,
        #[serde(default)]
        pub reason: String,
        #[serde(default)]
        pub status: String,
        #[serde(default)]
        pub created_at: Option<String>,
        #[serde(default)]
        pub note: Option<String>,
    }

    impl ModerationReport {
        pub fn status_typed(&self) -> ReportStatus {
            ReportStatus::from_wire(&self.status).unwrap_or(ReportStatus::Open)
        }

        pub fn is_resolvable(&self) -> bool {
            matches!(self.status_typed(), ReportStatus::Open) && !self.report_id.is_empty()
        }
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum ReportDecision {
        Resolve,
        Dismiss,
    }

    impl ReportDecision {
        pub fn label(&self) -> &'static str {
            match self {
                ReportDecision::Resolve => "resolve",
                ReportDecision::Dismiss => "dismiss",
            }
        }
    }

    /// OpenAPI schema: `ResolveReportRequest`.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResolveReportRequest {
        pub decision: ReportDecision,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub note: Option<String>,
    }

    pub type Policy = crate::types::api::Policy;
    pub type CreatePolicyRequest = crate::types::api::CreatePolicyRequest;
    pub type PolicyListResponse = crate::types::api::ListResponse<Policy>;
}

pub mod starid {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    /// OpenAPI schema: `IdentityDescribeResBody` from starid.
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
    use super::soland_admin::*;
    use super::starid::StaridDescribe;

    #[test]
    fn authz_generated_dto_round_trips_status() {
        let raw = r#"{
            "grant_id": "grant-1",
            "holder_did": "did:web:alice.example",
            "peer_did": "did:web:bob.example",
            "scope": "ck.cell.write",
            "status": "active"
        }"#;
        let grant: AuthzCapabilityGrant = serde_json::from_str(raw).unwrap();
        assert_eq!(grant.status_typed(), AuthzGrantStatus::Active);
        assert!(grant.is_revocable());
    }

    #[test]
    fn moderation_generated_request_serializes_snake_case() {
        let req = ResolveReportRequest {
            decision: ReportDecision::Dismiss,
            note: None,
        };
        let body = serde_json::to_string(&req).unwrap();
        assert!(body.contains("\"decision\":\"dismiss\""));
        assert!(!body.contains("\"note\""));
    }

    #[test]
    fn starid_generated_describe_tolerates_optional_status_fields() {
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
