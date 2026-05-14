//! DTO shapes for the per-account device admin surface
//!
//! Coauth owns the device list (one row per registered device per
//! account); soland owns the cascade-revoke that also revokes any
//! session grants tied to the device. The admin UI reads the list from
//! coauth and POSTs the revoke to coauth; soland is responsible for
//! the cascade soland-side (already wired, round 23).

use serde::{Deserialize, Serialize};

/// Lifecycle state for a single registered device. Mirrors coauth's
/// device reducer enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoauthDeviceStatus {
    Active,
    Stale,
    Revoked,
}

impl CoauthDeviceStatus {
    pub fn label(&self) -> &'static str {
        match self {
            CoauthDeviceStatus::Active => "Active",
            CoauthDeviceStatus::Stale => "Stale",
            CoauthDeviceStatus::Revoked => "Revoked",
        }
    }

    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "active" => Some(CoauthDeviceStatus::Active),
            "stale" => Some(CoauthDeviceStatus::Stale),
            "revoked" => Some(CoauthDeviceStatus::Revoked),
            _ => None,
        }
    }
}

/// One row in the per-account device list.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoauthDeviceRow {
    #[serde(default)]
    pub device_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    /// Wire-format `CoauthDeviceStatus` (snake_case).
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub last_seen_at: Option<String>,
    #[serde(default)]
    pub linked_session_count: u64,
    #[serde(default)]
    pub registered_at: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
}

impl CoauthDeviceRow {
    pub fn status_typed(&self) -> CoauthDeviceStatus {
        CoauthDeviceStatus::from_wire(&self.status).unwrap_or(CoauthDeviceStatus::Active)
    }

    /// Only `Active` and `Stale` devices can be revoked — re-revoking
    /// an already-revoked device would just be a noop on the server
    /// side, so we hide the button.
    pub fn is_revocable(&self) -> bool {
        !matches!(self.status_typed(), CoauthDeviceStatus::Revoked) && !self.device_id.is_empty()
    }
}

/// Format a relative "last-seen" cell. Pure helper — no access to
/// `Date.now()` — so it's just the wire string with a `-` fallback for
/// missing values. Kept separate from `signing_keys::render_*` so the
/// fallback rule is consistent across the device table.
pub fn render_last_seen(row: &CoauthDeviceRow) -> String {
    row.last_seen_at.clone().unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(status: &str, id: &str) -> CoauthDeviceRow {
        CoauthDeviceRow {
            device_id: id.into(),
            status: status.into(),
            ..Default::default()
        }
    }

    #[test]
    fn device_status_round_trips_via_wire_strings() {
        for (wire, label) in [
            ("active", "Active"),
            ("stale", "Stale"),
            ("revoked", "Revoked"),
        ] {
            let s = CoauthDeviceStatus::from_wire(wire).expect("variant");
            assert_eq!(s.label(), label);
        }
        assert!(CoauthDeviceStatus::from_wire("nope").is_none());
    }

    #[test]
    fn only_non_revoked_devices_with_id_are_revocable() {
        assert!(row("active", "d1").is_revocable());
        assert!(row("stale", "d1").is_revocable());
        assert!(!row("revoked", "d1").is_revocable());

        // Missing device_id hides the button regardless of status.
        let mut r = row("active", "d1");
        r.device_id.clear();
        assert!(!r.is_revocable());
    }

    #[test]
    fn last_seen_falls_back_to_dash_when_absent() {
        let mut r = row("active", "d1");
        assert_eq!(render_last_seen(&r), "-");
        r.last_seen_at = Some("2026-05-09T12:00:00Z".into());
        assert_eq!(render_last_seen(&r), "2026-05-09T12:00:00Z");
    }
}
