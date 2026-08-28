//! Generic client-side validating input component.
//!
//! Wraps the standard [`Input`] component with a configurable validator that
//! runs on every keystroke. Empty values are treated as untouched.

use arkret_identifiers::DidCoreId;
use dioxus::prelude::*;

use crate::components::ui::input::Input;
use crate::utils::i18n::t;

/// Validation rule enforced by [`ValidatedInput`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationKind {
    /// Stable Arkret identity grammar: `ak:did_core:<method>:<method-specific-id>`.
    DidCoreId,
}

impl ValidationKind {
    pub fn validate(&self, value: &str) -> Result<(), &'static str> {
        match self {
            ValidationKind::DidCoreId => {
                if value.is_empty() || DidCoreId::new(value.to_owned()).is_ok() {
                    Ok(())
                } else {
                    Err("did_core_id_input.invalid")
                }
            }
        }
    }
}

#[component]
pub fn ValidatedInput(
    #[props(default)] class: String,
    #[props(default)] placeholder: String,
    #[props(default = "text".to_string())] r#type: String,
    #[props(default)] disabled: bool,
    kind: ValidationKind,
    value: String,
    oninput: EventHandler<FormEvent>,
) -> Element {
    let trimmed = value.trim();
    let validation = kind.validate(trimmed);
    let show_error = !trimmed.is_empty();
    let error_msg = validation.err();

    rsx! {
        div { class: "space-y-1 {class}",
            Input {
                r#type,
                placeholder,
                value: value.clone(),
                disabled,
                oninput: move |evt| oninput.call(evt),
            }
            if show_error {
                if let Some(msg) = error_msg {
                    p { class: "text-xs text-destructive font-mono", {t(msg)} }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_passes_non_required_rules() {
        assert!(ValidationKind::DidCoreId.validate("").is_ok());
    }

    #[test]
    fn did_core_id_kind_accepts_only_stable_core_ids() {
        assert!(
            ValidationKind::DidCoreId
                .validate("ak:did_core:web:alice.example")
                .is_ok()
        );
        assert!(
            ValidationKind::DidCoreId
                .validate("did:web:alice.example")
                .is_err()
        );
    }
}
