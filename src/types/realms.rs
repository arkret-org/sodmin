//! DTO shapes for the Realm admin surface.

use arkret_wire::{Discoverability, JoinRule};
use serde::{Deserialize, Serialize};

// ── Realm types (security boundary) ──
//
// The object that carries encryption / join-rule / history-visibility /
// realm-class boundary fields is a Realm. Space containers are represented
// separately by `spaces::SpaceRow`.

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RealmClass {
    PrincipalControl,
    Collaboration,
}

impl RealmClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PrincipalControl => "principal_control",
            Self::Collaboration => "collaboration",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRealm {
    pub id: String,
    pub title: String,
    pub discoverability: Option<Discoverability>,
    pub created_by: Option<String>,
    pub member_count: u64,
    pub is_encrypted: bool,
    pub is_blocked: bool,
    pub topic: Option<String>,
    pub created_at: Option<String>,
    #[serde(rename = "default_join_rule")]
    pub join_rule: Option<JoinRule>,
    pub realm_class: Option<RealmClass>,
}

impl AdminRealm {
    pub fn discoverability_label(&self) -> Option<String> {
        self.discoverability.as_ref().map(wire_label)
    }

    pub fn join_rule_label(&self) -> Option<String> {
        self.join_rule.as_ref().map(wire_label)
    }

    pub fn type_label(&self) -> &'static str {
        self.realm_class.map(RealmClass::as_str).unwrap_or("-")
    }
}

fn wire_label<T>(value: &T) -> String
where
    T: Serialize,
{
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "-".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_realm() -> serde_json::Value {
        serde_json::json!({
            "id": "ak:realm:1",
            "title": "Realm",
            "member_count": 2,
            "is_encrypted": true,
            "is_blocked": false,
            "realm_class": "principal_control"
        })
    }

    #[test]
    fn realm_requires_producer_fields_and_closed_class() {
        let realm: AdminRealm = serde_json::from_value(valid_realm()).unwrap();
        assert_eq!(realm.realm_class, Some(RealmClass::PrincipalControl));
        assert_eq!(realm.type_label(), "principal_control");

        let mut missing = valid_realm();
        missing.as_object_mut().unwrap().remove("is_blocked");
        assert!(serde_json::from_value::<AdminRealm>(missing).is_err());

        let mut unknown = valid_realm();
        unknown["realm_class"] = serde_json::json!("PrincipalControl");
        assert!(serde_json::from_value::<AdminRealm>(unknown).is_err());
    }

    #[test]
    fn missing_optional_class_is_not_collaboration() {
        let mut value = valid_realm();
        value.as_object_mut().unwrap().remove("realm_class");
        let realm: AdminRealm = serde_json::from_value(value).unwrap();
        assert_eq!(realm.realm_class, None);
        assert_eq!(realm.type_label(), "-");
    }
}
