//! DTO shapes for the soland moderation reports admin surface
//! (Round 24, D5).
//!
//! Mirrors `GET /api/admin/v1/moderation/reports` (list open reports)
//! and `POST /api/admin/v1/moderation/reports/{id}/resolve` body
//! `{decision, note?}`. Hand-written for now; will become
//! `pub use soland_admin_types::*;` once the admin-types crate lands.

use serde::{Deserialize, Serialize};

/// Lifecycle state of a moderation report. The list endpoint defaults
/// to filtering for `open` rows; sodmin can pass `?status=…` to widen
/// the projection.
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

/// One row in the moderation reports admin panel.
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

    /// Resolve action is only available on Open rows that carry a
    /// stable `report_id`.
    pub fn is_resolvable(&self) -> bool {
        matches!(self.status_typed(), ReportStatus::Open) && !self.report_id.is_empty()
    }
}

/// Decision the operator made for a moderation report. `Resolve` =
/// the report was acted on and the matter is closed; `Dismiss` = the
/// report was reviewed and judged invalid / no-op.
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

/// Body POSTed to `/api/admin/v1/moderation/reports/{id}/resolve`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveReportRequest {
    pub decision: ReportDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(status: &str, id: &str) -> ModerationReport {
        ModerationReport {
            report_id: id.into(),
            status: status.into(),
            ..Default::default()
        }
    }

    #[test]
    fn report_status_round_trips() {
        for (wire, label) in [
            ("open", "Open"),
            ("resolved", "Resolved"),
            ("dismissed", "Dismissed"),
        ] {
            let s = ReportStatus::from_wire(wire).expect("variant");
            assert_eq!(s.label(), label);
        }
        assert!(ReportStatus::from_wire("nope").is_none());
    }

    #[test]
    fn only_open_reports_with_id_are_resolvable() {
        assert!(r("open", "r1").is_resolvable());

        let mut row = r("open", "r1");
        row.report_id.clear();
        assert!(!row.is_resolvable(), "missing report_id hides resolve");

        assert!(!r("resolved", "r1").is_resolvable());
        assert!(!r("dismissed", "r1").is_resolvable());
    }

    #[test]
    fn resolve_request_serializes_snake_case() {
        // The wire shape MUST be `{decision: "resolve" | "dismiss"}`.
        let req = ResolveReportRequest {
            decision: ReportDecision::Resolve,
            note: Some("ack".into()),
        };
        let s = serde_json::to_string(&req).unwrap();
        assert!(s.contains("\"decision\":\"resolve\""));
        assert!(s.contains("\"note\":\"ack\""));
        assert_eq!(ReportDecision::Resolve.label(), "resolve");

        let req = ResolveReportRequest {
            decision: ReportDecision::Dismiss,
            note: None,
        };
        let s = serde_json::to_string(&req).unwrap();
        assert!(s.contains("\"decision\":\"dismiss\""));
        // None note is dropped on the wire.
        assert!(!s.contains("\"note\""));
        assert_eq!(ReportDecision::Dismiss.label(), "dismiss");
    }
}
