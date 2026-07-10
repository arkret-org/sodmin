//! Round R2/R3 — Realm destroy confirmation dialog (T07).
//!
//! `ak.realm.destroy` is irreversible at the principal server: once
//! sealed, no further ordinary writes are accepted, all snapshots /
//! backfill / GC schedules collapse, federation fanout fires on a
//! 30-day window, and erasure receipts / legal holds take precedence
//! over backfill. The five normative bullets below MUST be explicitly
//! confirmed by the admin, *and* the admin must type the literal word
//! `DESTROY` into a confirmation input, before the destroy POST fires.
//!
//! This component is intentionally a leaf — it doesn't fetch or POST
//! anything, just renders the gating UI and emits a single `on_confirm`
//! event when all five checkboxes are ticked and the confirmation text
//! matches. The caller is responsible for the actual destroy API call.

use dioxus::prelude::*;

use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::Input;
use crate::components::ui::modal::ModalOverlay;
use crate::utils::i18n::t;

/// The exact string the admin must type into the confirmation field.
pub const CONFIRMATION_PHRASE: &str = "DESTROY";

/// Five normative bullets the admin must explicitly tick. Phrasing
/// (the dictionary values behind these keys) taken verbatim from the
/// round 2+3 spec.
const NORMATIVE_RULE_KEYS: [&str; 5] = [
    "realm_destroy_dialog.rule_no_ordinary_writes",
    "realm_destroy_dialog.rule_finalize_schedules",
    "realm_destroy_dialog.rule_no_successor_realm",
    "realm_destroy_dialog.rule_erasure_precedence",
    "realm_destroy_dialog.rule_federation_window",
];

#[derive(Props, Clone, PartialEq)]
pub struct RealmDestroyDialogProps {
    pub open: bool,
    pub realm_id: String,
    pub on_cancel: EventHandler<()>,
    pub on_confirm: EventHandler<()>,
}

#[component]
pub fn RealmDestroyDialog(props: RealmDestroyDialogProps) -> Element {
    let mut checked = use_signal(|| [false; 5]);
    let mut confirm_text = use_signal(String::new);

    if !props.open {
        return rsx! {};
    }

    let all_checked = checked.read().iter().all(|c| *c);
    let phrase_ok = confirm_text.read().trim() == CONFIRMATION_PHRASE;
    let can_destroy = all_checked && phrase_ok;
    let realm_id = props.realm_id.clone();

    rsx! {
        ModalOverlay { on_close: move |_| props.on_cancel.call(()),
            div { class: "relative z-50 w-full max-w-xl rounded-lg border glass-panel p-6 shadow-lg space-y-4",
                div { class: "space-y-1",
                    h2 { class: "text-lg font-semibold text-destructive", {t("realm_destroy_dialog.title")} }
                    p { class: "text-sm text-muted-foreground",
                        {t("realm_destroy_dialog.intro_prefix")}
                        span { class: "font-mono", "{realm_id}" }
                        {t("realm_destroy_dialog.intro_suffix")}
                    }
                }

                ul { class: "space-y-2",
                    for (idx, rule_key) in NORMATIVE_RULE_KEYS.iter().enumerate() {
                        {
                            let is_checked = checked.read()[idx];
                            let rule_text = t(rule_key);
                            rsx! {
                                li { class: "flex items-start gap-2 text-sm",
                                    input {
                                        r#type: "checkbox",
                                        class: "mt-1",
                                        checked: is_checked,
                                        onchange: move |evt| {
                                            let parsed = evt.value().parse::<bool>().unwrap_or(!is_checked);
                                            let mut current = *checked.read();
                                            current[idx] = parsed;
                                            checked.set(current);
                                        },
                                    }
                                    span { "{rule_text}" }
                                }
                            }
                        }
                    }
                }

                div { class: "space-y-1",
                    label { class: "text-sm font-medium",
                        {t("realm_destroy_dialog.type_prefix")}
                        span { class: "font-mono", "{CONFIRMATION_PHRASE}" }
                        {t("realm_destroy_dialog.type_suffix")}
                    }
                    Input {
                        r#type: "text".to_string(),
                        placeholder: CONFIRMATION_PHRASE.to_string(),
                        value: confirm_text.read().clone(),
                        oninput: move |evt: FormEvent| confirm_text.set(evt.value()),
                    }
                }

                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| props.on_cancel.call(()),
                        {t("realm_destroy_dialog.cancel")}
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        disabled: !can_destroy,
                        onclick: move |_| {
                            if can_destroy {
                                props.on_confirm.call(());
                            }
                        },
                        {t("realm_destroy_dialog.seal_confirm")}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_phrase_is_case_sensitive() {
        // The admin must type exactly "DESTROY". Lowercase / mixed
        // case are intentionally rejected to make the gate harder to
        // satisfy by accident.
        assert_eq!(CONFIRMATION_PHRASE, "DESTROY");
        assert_ne!(CONFIRMATION_PHRASE, "destroy");
    }

    #[test]
    fn five_normative_rules_present() {
        // Spec calls for five explicit bullets; if the list ever grows
        // / shrinks the spec is the source of truth. The canonical
        // phrasing lives in the i18n dictionaries under these keys.
        assert_eq!(NORMATIVE_RULE_KEYS.len(), 5);
        let unique: std::collections::BTreeSet<_> = NORMATIVE_RULE_KEYS.iter().collect();
        assert_eq!(unique.len(), NORMATIVE_RULE_KEYS.len());
        assert!(
            NORMATIVE_RULE_KEYS
                .iter()
                .all(|k| k.starts_with("realm_destroy_dialog.rule_"))
        );
    }
}
