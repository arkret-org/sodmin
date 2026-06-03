//! Round R2/R3 — Realm destroy confirmation dialog (T07).
//!
//! `ck.realm.destroy` is irreversible at the principal server: once
//! anchored, no further ordinary writes are accepted, all snapshots /
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

/// The exact string the admin must type into the confirmation field.
pub const CONFIRMATION_PHRASE: &str = "DESTROY";

/// Five normative bullets the admin must explicitly tick. Phrasing
/// taken verbatim from the round 2+3 spec.
const NORMATIVE_RULES: [&str; 5] = [
    "No further ordinary writes accepted after destroy is anchored.",
    "Snapshots, backfill, and GC schedules will run to finalize the destroy.",
    "No successor Realm — use a tombstone if continuity is required.",
    "Erasure receipts and legal holds take precedence over backfill.",
    "Federation fanout uses a 30-day window before remote PSes finalize.",
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
        div { class: "fixed inset-0 z-50 flex items-center justify-center",
            div {
                class: "fixed inset-0 bg-black/80",
                onclick: move |_| props.on_cancel.call(()),
            }
            div { class: "relative z-50 w-full max-w-xl rounded-lg border glass-panel p-6 shadow-lg space-y-4",
                div { class: "space-y-1",
                    h2 { class: "text-lg font-semibold text-destructive", "Destroy Realm" }
                    p { class: "text-sm text-muted-foreground",
                        "You are about to anchor a `ck.realm.destroy` event on "
                        span { class: "font-mono", "{realm_id}" }
                        ". This is irreversible."
                    }
                }

                ul { class: "space-y-2",
                    for (idx, rule) in NORMATIVE_RULES.iter().enumerate() {
                        {
                            let is_checked = checked.read()[idx];
                            let rule_text = (*rule).to_string();
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
                        "Type "
                        span { class: "font-mono", "{CONFIRMATION_PHRASE}" }
                        " to confirm:"
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
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        disabled: !can_destroy,
                        onclick: move |_| {
                            if can_destroy {
                                props.on_confirm.call(());
                            }
                        },
                        "Anchor ck.realm.destroy"
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
        // / shrinks the spec is the source of truth.
        assert_eq!(NORMATIVE_RULES.len(), 5);
        // Spot-check a few of the canonical phrases.
        assert!(
            NORMATIVE_RULES
                .iter()
                .any(|r| r.contains("ordinary writes"))
        );
        assert!(
            NORMATIVE_RULES
                .iter()
                .any(|r| r.contains("Erasure receipts"))
        );
        assert!(NORMATIVE_RULES.iter().any(|r| r.contains("30-day")));
    }
}
