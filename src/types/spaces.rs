//! DTO shapes for the soland Space container admin surface.
//!
//! Mirrors `GET /_soland/admin/spaces` (list). Per-space hierarchy is
//! assembled client-side from the snapshot's `parent_space_id` fields.

use serde::{Deserialize, Serialize};

/// Lifecycle badge for a single Space container row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "active" => Some(SpaceHealth::Active),
            "archived" => Some(SpaceHealth::Archived),
            "tombstoned" => Some(SpaceHealth::Tombstoned),
            _ => None,
        }
    }
}

/// One Space row in the admin list.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpaceRow {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub member_count: u64,
    /// Wire-format `SpaceHealth` (snake_case).
    #[serde(default)]
    pub health: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub parent_space_id: Option<String>,
}

impl SpaceRow {
    pub fn health_typed(&self) -> SpaceHealth {
        SpaceHealth::from_wire(&self.health).unwrap_or(SpaceHealth::Active)
    }
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
    fn space_health_wire_round_trip() {
        for (wire, label_key) in [
            ("active", "spaces.health_active"),
            ("archived", "spaces.health_archived"),
            ("tombstoned", "spaces.health_tombstoned"),
        ] {
            let h = SpaceHealth::from_wire(wire).expect("variant");
            assert_eq!(h.label(), label_key);
        }
        assert!(SpaceHealth::from_wire("nope").is_none());
    }

    #[test]
    fn space_row_health_typed_falls_back_to_active() {
        let r = SpaceRow {
            health: "garbage".into(),
            ..Default::default()
        };
        assert_eq!(r.health_typed(), SpaceHealth::Active);

        let r = SpaceRow {
            health: "archived".into(),
            ..Default::default()
        };
        assert_eq!(r.health_typed(), SpaceHealth::Archived);
    }
}
