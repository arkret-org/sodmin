//! DTO shapes for the push-route admin surface.

use serde::{Deserialize, Serialize};

// ── Push route / device route (T6.2 §4) ──

/// Surface for `ck.device.push_route` cells, grouped by principal so
/// the operator can inspect what each user is currently subscribed to
/// without leaking the raw `push_target_id`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PushRouteRow {
    #[serde(default)]
    pub principal_id: String,
    #[serde(default)]
    pub device_id: String,
    #[serde(default)]
    pub cell_subject: String,
    /// The 4-tuple `(principal_id, device_id, transport, route_id)`
    /// formatted for human display. Soland already emits this.
    #[serde(default)]
    pub cell_subject_tuple: Vec<String>,
    #[serde(default)]
    pub transport: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    /// Opaque, sensitive. The UI must keep this collapsed by default.
    #[serde(default)]
    pub push_target_id: Option<String>,
    #[serde(default)]
    pub last_rotation_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}
