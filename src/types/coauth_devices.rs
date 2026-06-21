//! DTO shapes for the per-account device admin surface.
//!
//! coauth owns the device list and cascade revocation. The admin UI reads the
//! account-scoped list from coauth and POSTs the revoke request back to coauth.

use serde::{Deserialize, Serialize};

/// Risk level reported by coauth for a registered device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoauthDeviceRiskLevel {
    Low,
    Medium,
    High,
    Unknown,
}

impl CoauthDeviceRiskLevel {
    pub fn label(&self) -> &'static str {
        match self {
            CoauthDeviceRiskLevel::Low => "Low",
            CoauthDeviceRiskLevel::Medium => "Medium",
            CoauthDeviceRiskLevel::High => "High",
            CoauthDeviceRiskLevel::Unknown => "Unknown",
        }
    }
}

/// MFA state reported by coauth for a registered device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoauthDeviceMfaState {
    Verified,
    Required,
    Unknown,
}

impl CoauthDeviceMfaState {
    pub fn label(&self) -> &'static str {
        match self {
            CoauthDeviceMfaState::Verified => "Verified",
            CoauthDeviceMfaState::Required => "Required",
            CoauthDeviceMfaState::Unknown => "Unknown",
        }
    }
}

/// One row in the coauth per-account device list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoauthDeviceRow {
    pub id: String,
    pub account_id: Option<String>,
    pub display_name: Option<String>,
    pub risk_level: CoauthDeviceRiskLevel,
    pub mfa_state: CoauthDeviceMfaState,
    pub registered_at: Option<String>,
    pub revoked_at: Option<String>,
}

impl CoauthDeviceRow {
    /// coauth represents revocation with `revoked_at`.
    pub fn is_revocable(&self) -> bool {
        !self.id.is_empty() && self.revoked_at.is_none()
    }
}

pub fn render_optional_timestamp(value: Option<&str>) -> String {
    value.unwrap_or("-").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, revoked_at: Option<&str>) -> CoauthDeviceRow {
        CoauthDeviceRow {
            id: id.into(),
            account_id: Some("01JZ9PK6HKFY0MM7C0TMZ1X8N7".into()),
            display_name: None,
            risk_level: CoauthDeviceRiskLevel::Unknown,
            mfa_state: CoauthDeviceMfaState::Unknown,
            registered_at: None,
            revoked_at: revoked_at.map(str::to_owned),
        }
    }

    #[test]
    fn labels_match_current_coauth_device_enums() {
        assert_eq!(CoauthDeviceRiskLevel::Low.label(), "Low");
        assert_eq!(CoauthDeviceRiskLevel::Medium.label(), "Medium");
        assert_eq!(CoauthDeviceRiskLevel::High.label(), "High");
        assert_eq!(CoauthDeviceRiskLevel::Unknown.label(), "Unknown");

        assert_eq!(CoauthDeviceMfaState::Verified.label(), "Verified");
        assert_eq!(CoauthDeviceMfaState::Required.label(), "Required");
        assert_eq!(CoauthDeviceMfaState::Unknown.label(), "Unknown");
    }

    #[test]
    fn only_unrevoked_devices_with_id_are_revocable() {
        assert!(row("d1", None).is_revocable());
        assert!(!row("d1", Some("2026-05-09T12:00:00Z")).is_revocable());
        assert!(!row("", None).is_revocable());
    }

    #[test]
    fn optional_timestamp_falls_back_to_dash_when_absent() {
        assert_eq!(render_optional_timestamp(None), "-");
        assert_eq!(
            render_optional_timestamp(Some("2026-05-09T12:00:00Z")),
            "2026-05-09T12:00:00Z"
        );
    }

    #[test]
    fn deserializes_current_coauth_device_record() {
        let row: CoauthDeviceRow = serde_json::from_value(serde_json::json!({
            "id": "device-1",
            "account_id": "01JZ9PK6HKFY0MM7C0TMZ1X8N7",
            "display_name": "Laptop",
            "risk_level": "high",
            "mfa_state": "required",
            "registered_at": "2026-05-09T12:00:00Z",
            "revoked_at": null
        }))
        .expect("current coauth DeviceRecord should deserialize");

        assert_eq!(row.id, "device-1");
        assert_eq!(row.risk_level, CoauthDeviceRiskLevel::High);
        assert_eq!(row.mfa_state, CoauthDeviceMfaState::Required);
        assert!(row.is_revocable());
    }
}
