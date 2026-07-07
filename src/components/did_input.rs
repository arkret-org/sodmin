//! Round 4 — DID-shaped input field with inline regex validation.
//!
//! Wraps the standard [`Input`] component and surfaces a local
//! validation error when the current value does not match the round-4
//! tightened DID grammar (`^did:[a-z0-9]+:[^\s]+$`). Empty values are
//! allowed (so the field can render before the operator types
//! anything); validation only fires on non-empty input.
//!
//! Use this in any admin form that needs a DID — handle reassignment,
//! capability grants, DID-binding edits, etc. The parent is responsible
//! for blocking submit when [`crate::utils::security::did::is_valid_did`] returns
//! `false` for the trimmed value.

use dioxus::prelude::*;

use crate::components::validated_input::{ValidatedInput, ValidationKind};

#[component]
pub fn DidInput(
    #[props(default)] class: String,
    #[props(default = "did:webvh:example".to_string())] placeholder: String,
    #[props(default)] disabled: bool,
    value: String,
    oninput: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        ValidatedInput {
            class,
            placeholder,
            disabled,
            kind: ValidationKind::Did,
            value,
            oninput,
        }
    }
}
