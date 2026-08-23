//! Generic client-side validating input component.
//!
//! Wraps the standard [`Input`] component with a configurable validator that
//! runs on every keystroke. Empty values are treated as untouched.

use arkret_identifiers::is_did;
use dioxus::prelude::*;

use crate::components::ui::input::Input;
use crate::utils::i18n::t;

/// Validation rule enforced by [`ValidatedInput`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationKind {
    /// Tight DID grammar: `^did:[a-z0-9]+:[^\s]+$`.
    Did,
}

impl ValidationKind {
    pub fn validate(&self, value: &str) -> Result<(), &'static str> {
        match self {
            ValidationKind::Did => {
                if value.is_empty() || is_did(value) {
                    Ok(())
                } else {
                    Err("did_input.invalid")
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
    fn did_kind_accepts_valid_did() {
        assert!(ValidationKind::Did.validate("did:web:alice").is_ok());
    }

    #[test]
    fn empty_passes_non_required_rules() {
        assert!(ValidationKind::Did.validate("").is_ok());
    }
}
