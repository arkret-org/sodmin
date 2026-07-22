//! Capability grant admin surface — contract-authoritative types plus thin
//! display helpers.
//!
//! The row is the shared `soland_contracts::admin::CapabilitySummary` (D14
//! production projection of the authz read-index grant). sodmin adds no
//! wire mirror; display-only conveniences live in [`CapabilitySummaryExt`].

pub use soland_contracts::admin::CapabilitySummary;

/// Display helpers for the shared [`CapabilitySummary`].
pub trait CapabilitySummaryExt {
    fn actions_display(&self) -> String;
    fn resource_display(&self) -> String;
}

impl CapabilitySummaryExt for CapabilitySummary {
    fn actions_display(&self) -> String {
        if self.actions.is_empty() {
            "-".to_string()
        } else {
            self.actions.join(", ")
        }
    }

    fn resource_display(&self) -> String {
        self.resource.clone().unwrap_or_else(|| "-".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{CapabilitySummary, CapabilitySummaryExt};

    #[test]
    fn summary_parses_production_wire_shape() {
        let summary: CapabilitySummary = serde_json::from_value(serde_json::json!({
            "grant_id": "ak:grant:01964137-0000-7000-8000-000000000001",
            "realm_id": "ak:realm:01964137-0000-7000-8000-0000000000aa",
            "issuer": "did:web:issuer.example",
            "subject": "did:web:subject.example",
            "resource": "realm",
            "actions": ["ak.message.create"],
            "revoked": false,
            "created_at": "2026-06-07T00:00:00.000Z"
        }))
        .expect("production-shaped grant summary should deserialize");

        assert_eq!(summary.issuer, "did:web:issuer.example");
        assert_eq!(summary.actions_display(), "ak.message.create");
        assert_eq!(summary.resource_display(), "realm");
        assert!(!summary.revoked);
    }
}
