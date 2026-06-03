//! DTO shapes for the component registry / version drift admin surface
//! (Stream H', H'6).
//!
//! Read from `GET /_soland/admin/components` — server-wide registry status:
//! the list of `ck.component.*` types the server knows about, the
//! cell_family they project into, the criticality classification (per spec
//! component-criticality table), the spec version pinned by the bundle and
//! the implementation version actually shipped. When `spec_version !=
//! impl_version` we surface a drift indicator.

use serde::{Deserialize, Serialize};

/// Criticality classification for a component. Per spec, this drives
/// recovery / takeover behavior — `Critical` components block startup if
/// drifted, `Important` warns, `Optional` is informational.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComponentCriticality {
    Critical,
    Important,
    Optional,
}

impl ComponentCriticality {
    pub fn label(&self) -> &'static str {
        match self {
            ComponentCriticality::Critical => "Critical",
            ComponentCriticality::Important => "Important",
            ComponentCriticality::Optional => "Optional",
        }
    }

    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "critical" => Some(ComponentCriticality::Critical),
            "important" => Some(ComponentCriticality::Important),
            "optional" => Some(ComponentCriticality::Optional),
            _ => None,
        }
    }
}

/// Implementation status the server reports for a component. `Active`
/// means the reducer/projector is wired; `Stub` means the registry knows
/// the type but the impl is not loaded; `Disabled` means an admin
/// explicitly turned it off.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComponentImplStatus {
    Active,
    Stub,
    Disabled,
}

impl ComponentImplStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ComponentImplStatus::Active => "Active",
            ComponentImplStatus::Stub => "Stub",
            ComponentImplStatus::Disabled => "Disabled",
        }
    }

    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "active" => Some(ComponentImplStatus::Active),
            "stub" => Some(ComponentImplStatus::Stub),
            "disabled" => Some(ComponentImplStatus::Disabled),
            _ => None,
        }
    }
}

/// One row in the component registry table.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentRegistryEntry {
    /// e.g. `ck.component.anchorer.v1`.
    pub component_type: String,
    /// e.g. `cas_register`, `or_set`, `lww_register`.
    pub cell_family: String,
    /// Wire-format `ComponentCriticality`.
    pub criticality: String,
    /// Spec version the server's contract is pinned at, e.g. `"v1.2"`.
    pub spec_version: String,
    /// Implementation version actually loaded by the server, e.g. `"v1.2"`.
    /// Empty string means "no impl reported".
    #[serde(default)]
    pub impl_version: String,
    /// Wire-format `ComponentImplStatus`.
    pub status: String,
    #[serde(default)]
    pub note: Option<String>,
}

/// Response from the `POST /_soland/admin/components/{type}/refresh`
/// route. soland reports the new resolved impl version (or echoes the
/// existing one when no refresh was needed) and a short status string.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentRefreshResponse {
    pub component_type: String,
    #[serde(default)]
    pub spec_version: String,
    #[serde(default)]
    pub impl_version: String,
    /// Short status: "refreshed" | "noop" | "queued". Passed through to
    /// the success toast so the operator sees what happened.
    #[serde(default)]
    pub status: Option<String>,
}

impl ComponentRegistryEntry {
    /// Drift = the server claims it conforms to `spec_version` but the
    /// loaded impl reports a different `impl_version`. Empty `impl_version`
    /// (the impl didn't report) is also drift — we don't know what loaded.
    pub fn has_drift(&self) -> bool {
        if self.impl_version.is_empty() {
            return true;
        }
        self.spec_version != self.impl_version
    }

    pub fn criticality_typed(&self) -> ComponentCriticality {
        ComponentCriticality::from_wire(&self.criticality).unwrap_or(ComponentCriticality::Optional)
    }

    pub fn status_typed(&self) -> ComponentImplStatus {
        ComponentImplStatus::from_wire(&self.status).unwrap_or(ComponentImplStatus::Stub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn criticality_round_trip() {
        for (wire, label) in [
            ("critical", "Critical"),
            ("important", "Important"),
            ("optional", "Optional"),
        ] {
            assert_eq!(
                ComponentCriticality::from_wire(wire).unwrap().label(),
                label
            );
        }
        assert!(ComponentCriticality::from_wire("nope").is_none());
    }

    #[test]
    fn impl_status_round_trip() {
        for (wire, label) in [
            ("active", "Active"),
            ("stub", "Stub"),
            ("disabled", "Disabled"),
        ] {
            assert_eq!(ComponentImplStatus::from_wire(wire).unwrap().label(), label);
        }
        assert!(ComponentImplStatus::from_wire("xx").is_none());
    }

    #[test]
    fn drift_detected_when_versions_differ() {
        let entry = ComponentRegistryEntry {
            component_type: "ck.component.anchorer.v1".into(),
            cell_family: "cas_register".into(),
            criticality: "critical".into(),
            spec_version: "v1.2".into(),
            impl_version: "v1.1".into(),
            status: "active".into(),
            ..Default::default()
        };
        assert!(entry.has_drift());
    }

    #[test]
    fn no_drift_when_versions_match() {
        let entry = ComponentRegistryEntry {
            spec_version: "v1.2".into(),
            impl_version: "v1.2".into(),
            ..Default::default()
        };
        assert!(!entry.has_drift());
    }

    #[test]
    fn missing_impl_version_counts_as_drift() {
        // "we don't know what's loaded" is itself a drift signal — the
        // admin should see a warning badge so they can investigate.
        let entry = ComponentRegistryEntry {
            spec_version: "v1.2".into(),
            impl_version: String::new(),
            ..Default::default()
        };
        assert!(entry.has_drift());
    }

    #[test]
    fn typed_accessors_have_safe_defaults() {
        let entry = ComponentRegistryEntry {
            criticality: "garbage".into(),
            status: "garbage".into(),
            ..Default::default()
        };
        // Unknown criticality collapses to Optional (least-alarming),
        // unknown status collapses to Stub.
        assert_eq!(entry.criticality_typed(), ComponentCriticality::Optional);
        assert_eq!(entry.status_typed(), ComponentImplStatus::Stub);
    }
}
