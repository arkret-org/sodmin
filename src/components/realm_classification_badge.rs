//! P3A.6 — visual badge that distinguishes "Principal Control" Realms
//! (those that back DID issuance / recovery / cross-signing) from
//! ordinary "Collaboration" Realms.
//!
//! Per CXP-0007 the classification is set at create time and is
//! immutable afterwards. Most Realm surfaces in sodmin (member panels,
//! delivery-binding editor, audit drilldown) should render this badge
//! so operators can tell at a glance whether an action will touch
//! identity-bearing infrastructure or just collaboration scope.

use dioxus::prelude::*;

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::utils::i18n::t;

/// The two CXP-0007 Realm classes. Carried as a free-form string on
/// the wire (`"principal_control"` / `"collaboration"`) so newer
/// values (e.g. a future `"federation"` class) keep deserialising;
/// anything unknown falls back to a muted outline badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealmClass {
    PrincipalControl,
    Collaboration,
    Unknown,
}

impl RealmClass {
    pub fn from_wire(raw: &str) -> Self {
        match raw {
            "principal_control" | "PrincipalControl" => Self::PrincipalControl,
            "collaboration" | "Collaboration" => Self::Collaboration,
            _ => Self::Unknown,
        }
    }
}

#[component]
pub fn RealmClassificationBadge(#[props(default)] class: String, realm_class: String) -> Element {
    let parsed = RealmClass::from_wire(&realm_class);
    let (variant, key) = match parsed {
        RealmClass::PrincipalControl => (BadgeVariant::Destructive, "realm.principal_control"),
        RealmClass::Collaboration => (BadgeVariant::Secondary, "realm.collaboration"),
        RealmClass::Unknown => (BadgeVariant::Outline, "realm.classification"),
    };
    rsx! {
        Badge { variant, class: class.clone(), {t(key)} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realm_class_from_wire_matches_spec_strings() {
        assert_eq!(
            RealmClass::from_wire("principal_control"),
            RealmClass::PrincipalControl
        );
        assert_eq!(
            RealmClass::from_wire("collaboration"),
            RealmClass::Collaboration
        );
        assert_eq!(RealmClass::from_wire("wat"), RealmClass::Unknown);
    }
}
