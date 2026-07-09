//! Reusable confirmation dialog gated on a typed phrase.
//!
//! Unlike [`crate::components::ui::dialog::ConfirmDialog`], this dialog
//! REQUIRES the admin to type a specific phrase verbatim before the
//! destructive button enables. The phrase is parameterised so each call
//! site can pick a value that's distinctive but cheap to type:
//!
//! * device revoke -> the last 4 chars of the device ID
//! * applet suspend/revoke -> the first 6 chars of the applet name
//! * realm destroy -> the literal word `DESTROY` (see [`crate::components::realm_destroy_dialog`]
//!   for the heavier variant with five normative-bullet checkboxes)
//!
//! Pure leaf component — it does not fetch or POST anything. The caller
//! is responsible for the actual mutation when `on_confirm` fires.

use dioxus::prelude::*;

use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::Input;
use crate::components::ui::modal::ModalOverlay;
use crate::utils::i18n::t;

#[derive(Props, Clone, PartialEq)]
pub struct DangerousActionDialogProps {
    pub open: bool,
    pub title: String,
    pub description: String,
    /// Exact phrase the admin must type. Matching is whitespace-trimmed
    /// and case-sensitive so an accidental shift-key press doesn't
    /// satisfy the gate.
    pub confirmation_phrase: String,
    #[props(default = "Confirm".to_string())]
    pub confirm_text: String,
    #[props(default = "Cancel".to_string())]
    pub cancel_text: String,
    pub on_cancel: EventHandler<()>,
    pub on_confirm: EventHandler<()>,
}

#[component]
pub fn DangerousActionDialog(props: DangerousActionDialogProps) -> Element {
    let mut typed = use_signal(String::new);

    // Reset the typed buffer whenever `open` flips off OR the
    // required phrase changes (i.e. operator picks a different row),
    // so a stale buffer never satisfies a new gate. `use_effect`
    // reacts to prop changes without mutating signals during render.
    let open_flag = props.open;
    let phrase_for_effect = props.confirmation_phrase.clone();
    use_effect(move || {
        // Touch the deps so the effect re-runs when either changes.
        let _ = (open_flag, phrase_for_effect.clone());
        typed.set(String::new());
    });

    if !props.open {
        return rsx! {};
    }

    let phrase = props.confirmation_phrase.clone();
    let phrase_ok = phrase_matches(&typed.read(), &phrase);

    rsx! {
        ModalOverlay { on_close: move |_| props.on_cancel.call(()),
            div { class: "relative z-50 w-full max-w-lg rounded-lg border glass-panel p-6 shadow-lg space-y-4",
                div { class: "space-y-1",
                    h2 { class: "text-lg font-semibold text-destructive", "{props.title}" }
                    p { class: "text-sm text-muted-foreground", "{props.description}" }
                }

                div { class: "space-y-1",
                    label { class: "text-sm font-medium",
                        {t("dangerous_action.type_prefix")}
                        span { class: "font-mono", "{phrase}" }
                        {t("dangerous_action.type_suffix")}
                    }
                    Input {
                        r#type: "text".to_string(),
                        placeholder: phrase.clone(),
                        value: typed.read().clone(),
                        oninput: move |evt: FormEvent| typed.set(evt.value()),
                    }
                }

                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| props.on_cancel.call(()),
                        "{props.cancel_text}"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        disabled: !phrase_ok,
                        onclick: move |_| {
                            if phrase_ok {
                                props.on_confirm.call(());
                            }
                        },
                        "{props.confirm_text}"
                    }
                }
            }
        }
    }
}

/// Pure helper: case-sensitive, whitespace-trimmed equality of the
/// typed buffer against the required phrase. Empty phrases never
/// satisfy the gate (defensive — the caller has nothing to confirm
/// against).
pub fn phrase_matches(typed: &str, phrase: &str) -> bool {
    if phrase.is_empty() {
        return false;
    }
    typed.trim() == phrase
}

/// Pure helper: pick the last `n` chars of a stable identifier for a typed
/// confirmation gate. If the id is shorter than `n`, the whole id is returned.
pub fn confirmation_suffix(id: &str, n: usize) -> String {
    let chars: Vec<char> = id.chars().collect();
    if chars.len() <= n {
        id.to_string()
    } else {
        chars[chars.len() - n..].iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phrase_match_is_case_sensitive() {
        assert!(phrase_matches("DESTROY", "DESTROY"));
        assert!(!phrase_matches("destroy", "DESTROY"));
    }

    #[test]
    fn phrase_match_trims_surrounding_whitespace() {
        assert!(phrase_matches("  DESTROY \n", "DESTROY"));
    }

    #[test]
    fn empty_phrase_never_matches() {
        assert!(!phrase_matches("", ""));
        assert!(!phrase_matches("anything", ""));
    }

    #[test]
    fn device_phrase_picks_last_n_chars() {
        assert_eq!(confirmation_suffix("dev_01HXYAB7K9", 4), "B7K9");
        // shorter than n -> whole id
        assert_eq!(confirmation_suffix("ab", 4), "ab");
        // multibyte safe
        assert_eq!(confirmation_suffix("dev_\u{4e2d}\u{6587}id", 2), "id");
    }
}
