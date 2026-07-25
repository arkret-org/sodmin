//! DTO shapes for the soland Space container admin surface.
//!
//! Mirrors `GET /_soland/admin/spaces` (list). Per-space hierarchy is
//! assembled client-side from the snapshot's `parent_space_id` fields.

use serde::{Deserialize, Serialize};

/// Lifecycle badge for a single Space container row.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpaceHealth {
    Active,
    Archived,
    Tombstoned,
}

impl SpaceHealth {
    /// Returns the i18n key for the lifecycle label. Render via
    /// `crate::utils::i18n::t(health.label())` at the call site.
    pub fn label(&self) -> &'static str {
        match self {
            SpaceHealth::Active => "spaces.health_active",
            SpaceHealth::Archived => "spaces.health_archived",
            SpaceHealth::Tombstoned => "spaces.health_tombstoned",
        }
    }
}

/// One Space row in the admin list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceRow {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub member_count: u64,
    pub health: SpaceHealth,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub parent_space_id: Option<String>,
}

/// One node in the hierarchy tree derived from the admin Spaces snapshot.
/// `parent` is at most one step up; `children` is the full set of
/// immediate children found in the loaded snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpaceHierarchy {
    /// The Space this hierarchy is centered on.
    #[serde(default)]
    pub space_id: String,
    /// The display name for the centered Space (when soland populated
    /// it).
    #[serde(default)]
    pub name: Option<String>,
    /// Single parent (Spaces have at most one immediate parent).
    #[serde(default)]
    pub parent: Option<SpaceHierarchyNode>,
    /// Direct children (no transitive descendants).
    #[serde(default)]
    pub children: Vec<SpaceHierarchyNode>,
}

/// A single neighbouring node — kept compact since the full Space row
/// lives behind the detail page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SpaceHierarchyNode {
    #[serde(default)]
    pub space_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub member_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_health_wire_is_strict() {
        for (wire, expected, label_key) in [
            ("active", SpaceHealth::Active, "spaces.health_active"),
            ("archived", SpaceHealth::Archived, "spaces.health_archived"),
            (
                "tombstoned",
                SpaceHealth::Tombstoned,
                "spaces.health_tombstoned",
            ),
        ] {
            let h: SpaceHealth = serde_json::from_str(&format!("\"{wire}\"")).expect("variant");
            assert_eq!(h, expected);
            assert_eq!(h.label(), label_key);
        }
        assert!(serde_json::from_str::<SpaceHealth>("\"nope\"").is_err());
    }

    #[test]
    fn space_row_requires_known_health() {
        let base = serde_json::json!({
            "id": "space-1",
            "name": "Space",
            "member_count": 1,
            "health": "archived"
        });
        let row: SpaceRow = serde_json::from_value(base.clone()).unwrap();
        assert_eq!(row.health, SpaceHealth::Archived);

        let mut unknown = base.clone();
        unknown["health"] = serde_json::json!("garbage");
        assert!(serde_json::from_value::<SpaceRow>(unknown).is_err());

        let mut missing = base;
        missing.as_object_mut().unwrap().remove("health");
        assert!(serde_json::from_value::<SpaceRow>(missing).is_err());
    }
}
