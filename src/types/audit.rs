//! DTO shapes for the audit admin surface.

use serde::{Deserialize, Serialize};

// ── Audit types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditEntry {
    #[serde(default, alias = "audit_id")]
    pub id: String,
    #[serde(default)]
    pub action: String,
    #[serde(default, alias = "actor")]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub target_type: Option<String>,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default, alias = "payload")]
    pub details: Option<serde_json::Value>,
    #[serde(default, alias = "created_at")]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    /// CKP-0007 — the effective scope at which the action took effect
    /// (`ck:realm:...` or `ck:circle:...`). Distinct from the audited
    /// `target_id` because Circle actions surface inside a Realm
    /// envelope but get pinned to the Circle for replay-locality.
    /// `None` when the source row omits the field.
    #[serde(default)]
    pub effective_scope: Option<String>,
    /// CKP-0007 — when `effective_scope` points at a Circle, this is
    /// the parent realm id so the audit row can render a "jump to
    /// Realm" link without an extra round trip.
    #[serde(default)]
    pub scope_realm_id: Option<String>,
    /// CKP-0007 — convenience copy of `effective_scope` when it is a
    /// `ck:circle:...` id; saves the row a string-prefix sniff on
    /// the rendering path.
    #[serde(default)]
    pub scope_circle_id: Option<String>,
    /// CKP-0008 — when the envelope was signed/executed on behalf of
    /// the principal, this records the executing DID (e.g. a personal
    /// agent acting on behalf of the controller). Conditional: present
    /// only on agent-attributed envelopes.
    #[serde(default)]
    pub executed_by: Option<String>,
    /// CKP-0008 — typed id of the `accountability_grant` or capability
    /// grant whose validity authorized the action. Lets the audit row
    /// link back to the grant ledger row.
    #[serde(default)]
    pub authorization_ref: Option<String>,
    /// CKP-0008 — reducer-stamped projection of the actor classification
    /// at the moment of admission. Parsed with the SDK `ActorKind` enum,
    /// so unknown/non-registry values fail instead of being rendered as
    /// arbitrary strings.
    #[serde(default)]
    pub actor_kind: Option<cokret_core::models::ActorKind>,
}

impl AuditEntry {
    /// Classify the audit entry's effective scope for badge / link
    /// rendering. Pure helper so the rule stays unit-testable.
    pub fn scope_kind(&self) -> AuditScopeKind {
        if let Some(ref s) = self.scope_circle_id
            && !s.is_empty()
        {
            return AuditScopeKind::Circle(s.clone());
        }
        if let Some(ref s) = self.effective_scope {
            if s.starts_with("ck:circle:") {
                return AuditScopeKind::Circle(s.clone());
            }
            if s.starts_with("ck:realm:") {
                return AuditScopeKind::Realm(s.clone());
            }
        }
        if let Some(ref r) = self.scope_realm_id
            && !r.is_empty()
        {
            return AuditScopeKind::Realm(r.clone());
        }
        AuditScopeKind::Unknown
    }
}

/// Discriminated effective-scope value for the audit views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditScopeKind {
    Realm(String),
    Circle(String),
    Unknown,
}
