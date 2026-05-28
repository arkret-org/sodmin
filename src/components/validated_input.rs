//! P5 — generic client-side validating input component.
//!
//! Wraps the standard [`Input`] component with a configurable validator
//! that runs on every keystroke. When the trimmed value fails validation
//! an inline error message is rendered underneath the field. Empty
//! values are treated as "untouched" — the parent is responsible for
//! flagging a missing required field via the [`ValidationKind::Required`]
//! variant which fires when the trimmed value is empty.
//!
//! Wired into:
//! * [`crate::pages::agents::personal::ProvisionWizard`] step 1 — the
//!   `controller_did` field uses [`ValidationKind::Did`].
//! * [`crate::components::did_binding_panel::DidBindingPanel`] — the
//!   `Add binding` form uses [`ValidationKind::Did`] for the DID and
//!   [`ValidationKind::Required`] for the control_proof opaque blob.
//!
use dioxus::prelude::*;

use crate::components::ui::input::Input;
use crate::utils::did;

/// Validation rule enforced by [`ValidatedInput`]. Each variant maps to
/// a single, well-known invariant — composite rules should layer two
/// inputs or use [`ValidationKind::MaxLength`] alongside a primary rule
/// by checking `value.len()` on submit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationKind {
    /// Tightened round-4 DID grammar — `^did:[a-z0-9]+:[^\s]+$`.
    Did,
    /// Trimmed value MUST be non-empty.
    Required,
    /// Trimmed value MUST be at most `n` characters long. Useful for
    /// display-name fields where the backend caps length at 128.
    MaxLength(usize),
}

impl ValidationKind {
    /// Validate `value` (already trimmed by the caller). Returns
    /// `Ok(())` on success, or an i18n key + fallback English message
    /// describing the failure. Empty input passes every rule except
    /// [`ValidationKind::Required`] so a "pristine" form does not
    /// flash red.
    pub fn validate(&self, value: &str) -> Result<(), &'static str> {
        match self {
            ValidationKind::Did => {
                if value.is_empty() || did::is_valid_did(value) {
                    Ok(())
                } else {
                    Err("DID must match did:<method>:<id> (round-4 grammar)")
                }
            }
            ValidationKind::Required => {
                if value.is_empty() {
                    Err("This field is required")
                } else {
                    Ok(())
                }
            }
            ValidationKind::MaxLength(max) => {
                if value.chars().count() <= *max {
                    Ok(())
                } else {
                    Err("Value exceeds the maximum allowed length")
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
    /// The validation rule applied to every input value. Multiple rules
    /// can be modelled by stacking two `ValidatedInput`s or by enforcing
    /// composite rules in the parent form's submit handler.
    kind: ValidationKind,
    value: String,
    oninput: EventHandler<FormEvent>,
) -> Element {
    let trimmed = value.trim();
    // Validation triggers only when the user has typed something; an
    // untouched, empty field stays clean unless the kind is
    // `Required`. `Required` fires inline because the parent typically
    // disables the submit button on a missing required field.
    let validation = kind.validate(trimmed);
    let show_error = !trimmed.is_empty() || matches!(kind, ValidationKind::Required);
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
                    p { class: "text-xs text-destructive font-mono", "{msg}" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn did_kind_accepts_round4_did() {
        assert!(ValidationKind::Did.validate("did:web:alice").is_ok());
    }

    #[test]
    fn did_kind_rejects_legacy_method_segment() {
        assert!(ValidationKind::Did.validate("did:web.vh:alice").is_err());
    }

    #[test]
    fn required_kind_rejects_empty() {
        assert!(ValidationKind::Required.validate("").is_err());
        assert!(ValidationKind::Required.validate("anything").is_ok());
    }

    #[test]
    fn max_length_rejects_overrun() {
        let kind = ValidationKind::MaxLength(3);
        assert!(kind.validate("abc").is_ok());
        assert!(kind.validate("abcd").is_err());
        // Unicode counted by chars, not bytes.
        assert!(kind.validate("中文测").is_ok());
        assert!(kind.validate("中文测试").is_err());
    }

    #[test]
    fn empty_passes_non_required_rules() {
        assert!(ValidationKind::Did.validate("").is_ok());
        assert!(ValidationKind::MaxLength(5).validate("").is_ok());
    }
}
