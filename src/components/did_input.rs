//! Stable Arkret identity input with inline validation.
//!
//! Wraps the standard [`Input`] component and surfaces a local
//! validation error when the current value is not an
//! [`arkret_identifiers::DidCoreId`]. Empty values are
//! allowed (so the field can render before the operator types
//! anything); validation only fires on non-empty input.
//!
//! The parent remains responsible for parsing the trimmed value into a
//! [`arkret_identifiers::DidCoreId`] before submitting it.

use dioxus::prelude::*;

use crate::components::validated_input::{ValidatedInput, ValidationKind};

#[component]
pub fn DidCoreIdInput(
    #[props(default)] class: String,
    #[props(default = "ak:did_core:web:example".to_string())] placeholder: String,
    #[props(default)] disabled: bool,
    value: String,
    oninput: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        ValidatedInput {
            class,
            placeholder,
            disabled,
            kind: ValidationKind::DidCoreId,
            value,
            oninput,
        }
    }
}
