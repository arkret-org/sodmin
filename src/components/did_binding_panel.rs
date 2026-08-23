//! DID-binding admin surface used on the coauth account detail page.
//!
//! Renders the current set of `managed_dids` and a `Remove binding`
//! confirmation modal.
//!
//! `on_mutated` is invoked on success so the parent can `data.restart()`
//! its snapshot resource and pull the fresh list back from the server.

use dioxus::prelude::*;

use crate::api::coauth::{self, CoauthManagedDidBinding};
use crate::components::dangerous_action_dialog::{DangerousActionDialog, confirmation_suffix};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::utils::i18n::t;
use crate::utils::net::error::HttpError;

#[component]
pub fn DidBindingPanel(
    account_id: String,
    bindings: Vec<CoauthManagedDidBinding>,
    on_mutated: EventHandler<()>,
) -> Element {
    // `Some(did)` when the user has clicked "Remove" on a row but not yet
    // confirmed the modal.
    let mut pending_remove = use_signal(|| None::<String>);
    let mut remove_in_flight = use_signal(|| false);
    let mut remove_error = use_signal(|| None::<String>);

    rsx! {
        div { class: "rounded-lg border p-4 space-y-3",
            h2 { class: "text-base font-semibold", {t("did_binding_panel.title")} }

            if bindings.is_empty() {
                p { class: "text-sm text-muted-foreground",
                    {t("did_binding_panel.empty")}
                }
            } else {
                ul { class: "space-y-2",
                    for binding in bindings.iter() {
                        {
                            let did_for_button = binding.did.clone();
                            let did_display = binding.did.clone();
                            let kind = binding.kind.label().to_string();
                            let state = binding.state.label().to_string();
                            let verification = binding.verification_status.label().to_string();
                            let primary = if binding.primary {
                                t("did_binding_panel.primary_yes")
                            } else {
                                t("did_binding_panel.primary_no")
                            };
                            let verified_at = binding.last_verified_at.clone().unwrap_or_else(|| "-".to_string());
                            rsx! {
                                li { class: "rounded-md border p-3 space-y-2",
                                    div { class: "flex items-start justify-between gap-3",
                                        div { class: "font-mono text-sm break-all", "{did_display}" }
                                        Button {
                                            variant: ButtonVariant::Destructive,
                                            onclick: move |_| {
                                                pending_remove.set(Some(did_for_button.clone()));
                                                remove_error.set(None);
                                            },
                                            {t("did_binding_panel.remove")}
                                        }
                                    }
                                    div { class: "grid gap-2 text-sm md:grid-cols-5",
                                        div {
                                            span { class: "text-muted-foreground", {t("did_binding_panel.label_kind")} }
                                            span { "{kind}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground", {t("did_binding_panel.label_state")} }
                                            span { class: "font-mono", "{state}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground", {t("did_binding_panel.label_verification")} }
                                            span { class: "font-mono", "{verification}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground", {t("did_binding_panel.label_primary")} }
                                            span { "{primary}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground", {t("did_binding_panel.label_last_verified")} }
                                            span { "{verified_at}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(err) = remove_error.read().clone() {
                div { class: "rounded-md bg-destructive/10 p-2 text-sm text-destructive",
                    "{err}"
                }
            }

            DangerousActionDialog {
                open: pending_remove.read().is_some(),
                confirmation_phrase: confirmation_suffix(pending_remove.read().as_deref().unwrap_or(""), 4),
                title: t("did_binding_panel.remove_title"),
                description: pending_remove
                    .read()
                    .clone()
                    .map(|d| t("did_binding_panel.remove_description").replace("{did}", &d))
                    .unwrap_or_default(),
                confirm_text: if *remove_in_flight.read() {
                    t("did_binding_panel.removing")
                } else {
                    t("did_binding_panel.remove")
                },
                cancel_text: t("did_binding_panel.cancel"),
                on_confirm: {
                    let account_id = account_id.clone();
                    move |_| {
                        let account_id = account_id.clone();
                        let did = match pending_remove.read().clone() {
                            Some(d) => d,
                            None => return,
                        };
                        spawn(async move {
                            remove_in_flight.set(true);
                            remove_error.set(None);
                            match coauth::remove_account_did_binding(&account_id, &did).await {
                                Ok(()) => {
                                    remove_in_flight.set(false);
                                    pending_remove.set(None);
                                    on_mutated.call(());
                                }
                                Err(e) => {
                                    remove_in_flight.set(false);
                                    pending_remove.set(None);
                                    remove_error.set(Some(format_err(&e)));
                                }
                            }
                        });
                    }
                },
                on_cancel: move |_| {
                    if !*remove_in_flight.read() {
                        pending_remove.set(None);
                    }
                },
            }
        }
    }
}

fn format_err(e: &HttpError) -> String {
    match e.status {
        0 => t("did_binding_panel.error_network").replace("{message}", &e.message),
        s => t("did_binding_panel.error_http")
            .replace("{status}", &s.to_string())
            .replace("{message}", &e.message),
    }
}
