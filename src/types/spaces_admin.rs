//! DTO shapes for the soland Spaces admin surface.
//!
//! Mirrors `GET /api/admin/v1/spaces` (list) and
//! `GET /api/admin/v1/spaces/{id}/hierarchy` (per-space hierarchy).

use serde::{Deserialize, Serialize};

/// Health badge for a single Space row. `Active` is the happy path
/// (Move/Anchor accepting writes); `Frozen` means the space is
/// quarantined (admin-induced or replication lag); `Destroyed` means a
/// `cx.cell.space.tombstone` Move has landed and the space is in the
/// tombstone-period for audit reads.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpaceHealth {
    Active,
    Frozen,
    Destroyed,
}

impl SpaceHealth {
    pub fn label(&self) -> &'static str {
        match self {
            SpaceHealth::Active => "Active",
            SpaceHealth::Frozen => "Frozen",
            SpaceHealth::Destroyed => "Destroyed",
        }
    }

    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "active" => Some(SpaceHealth::Active),
            "frozen" => Some(SpaceHealth::Frozen),
            "destroyed" => Some(SpaceHealth::Destroyed),
            _ => None,
        }
    }
}

/// One Space row in the admin list.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpaceAdminRow {
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

impl SpaceAdminRow {
    pub fn health_typed(&self) -> SpaceHealth {
        SpaceHealth::from_wire(&self.health).unwrap_or(SpaceHealth::Active)
    }
}

/// One node in the hierarchy tree returned by
/// `GET /api/admin/v1/spaces/{id}/hierarchy`. `parent` is at most one
/// step up; `children` is the full set of immediate children. Deeper
/// transitive ancestry must be paginated by following each parent in a
/// follow-up request.
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
        for (wire, label) in [
            ("active", "Active"),
            ("frozen", "Frozen"),
            ("destroyed", "Destroyed"),
        ] {
            let h = SpaceHealth::from_wire(wire).expect("variant");
            assert_eq!(h.label(), label);
        }
        assert!(SpaceHealth::from_wire("nope").is_none());
    }

    #[test]
    fn space_admin_row_health_typed_falls_back_to_active() {
        let r = SpaceAdminRow {
            health: "garbage".into(),
            ..Default::default()
        };
        assert_eq!(r.health_typed(), SpaceHealth::Active);

        let r = SpaceAdminRow {
            health: "frozen".into(),
            ..Default::default()
        };
        assert_eq!(r.health_typed(), SpaceHealth::Frozen);
    }
}
