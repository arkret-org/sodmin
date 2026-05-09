//! Claims admin lifecycle surface used on the coauth account detail
//! page. Lists current claims and exposes a `Revoke claim` button that
//! posts to the coauth claims revoke endpoint via
//! `api::coauth::revoke_account_claim`.
//!
//! Like `did_binding_panel`, the parent passes an `on_mutated` callback
//! so it can re-fetch the account snapshot after a successful revoke.

use dioxus::prelude::*;

use crate::api::coauth::{self, CoauthAccountClaim};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::utils::error::HttpError;

#[component]
pub fn ClaimsPanel(
    account_id: String,
    claims: Vec<CoauthAccountClaim>,
    on_mutated: EventHandler<()>,
) -> Element {
    let mut pending_revoke = use_signal(|| None::<String>);
    let mut revoke_in_flight = use_signal(|| false);
    let mut revoke_error = use_signal(|| None::<String>);

    rsx! {
        div { class: "rounded-lg border p-4 space-y-3",
            h2 { class: "text-base font-semibold", "Claims" }

            if claims.is_empty() {
                p { class: "text-sm text-muted-foreground",
                    "No claim material is exposed by the current coauth account admin contract."
                }
            } else {
                ul { class: "space-y-2",
                    for claim in claims.iter() {
                        {
                            let claim_type_for_button = claim.claim_type.clone();
                            let claim_type_display = claim.claim_type.clone();
                            let value = claim.value.clone().unwrap_or_else(|| "-".to_string());
                            let state = claim.state.clone().unwrap_or_else(|| "-".to_string());
                            let source = claim.source.clone().unwrap_or_else(|| "-".to_string());
                            // A claim that the server has already
                            // marked revoked should not show the revoke
                            // button — admins double-revoking is just
                            // noise.
                            let already_revoked = claim
                                .state
                                .as_deref()
                                .is_some_and(|s| s.contains("revoked"));
                            rsx! {
                                li { class: "rounded-md border p-3 space-y-2",
                                    div { class: "flex items-start justify-between gap-3",
                                        div { class: "font-medium", "{claim_type_display}" }
                                        if !already_revoked {
                                            Button {
                                                variant: ButtonVariant::Destructive,
                                                onclick: move |_| {
                                                    pending_revoke.set(Some(claim_type_for_button.clone()));
                                                    revoke_error.set(None);
                                                },
                                                "Revoke"
                                            }
                                        } else {
                                            span { class: "text-xs text-muted-foreground", "already revoked" }
                                        }
                                    }
                                    div { class: "grid gap-2 text-sm md:grid-cols-3",
                                        div {
                                            span { class: "text-muted-foreground", "Value: " }
                                            span { class: "break-all", "{value}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground", "State: " }
                                            span { class: "font-mono", "{state}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground", "Source: " }
                                            span { class: "font-mono", "{source}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(err) = revoke_error.read().clone() {
                div { class: "rounded-md bg-destructive/10 p-2 text-sm text-destructive",
                    "{err}"
                }
            }

            ConfirmDialog {
                open: pending_revoke.read().is_some(),
                title: "Revoke claim?".to_string(),
                description: pending_revoke
                    .read()
                    .clone()
                    .map(|c| format!(
                        "This will revoke the {} claim on this account via coauth's claims revoke endpoint.",
                        c
                    ))
                    .unwrap_or_default(),
                confirm_text: if *revoke_in_flight.read() { "Revoking...".to_string() } else { "Revoke".to_string() },
                cancel_text: "Cancel".to_string(),
                destructive: true,
                on_confirm: {
                    let account_id = account_id.clone();
                    move |_| {
                        let account_id = account_id.clone();
                        let claim_type = match pending_revoke.read().clone() {
                            Some(c) => c,
                            None => return,
                        };
                        spawn(async move {
                            revoke_in_flight.set(true);
                            revoke_error.set(None);
                            match coauth::revoke_account_claim(&account_id, &claim_type).await {
                                Ok(()) => {
                                    revoke_in_flight.set(false);
                                    pending_revoke.set(None);
                                    on_mutated.call(());
                                }
                                Err(e) => {
                                    revoke_in_flight.set(false);
                                    pending_revoke.set(None);
                                    revoke_error.set(Some(format_err(&e)));
                                }
                            }
                        });
                    }
                },
                on_cancel: move |_| {
                    if !*revoke_in_flight.read() {
                        pending_revoke.set(None);
                    }
                },
            }
        }
    }
}

fn format_err(e: &HttpError) -> String {
    match e.status {
        0 => format!("Network error: {}", e.message),
        s => format!("HTTP {}: {}", s, e.message),
    }
}
