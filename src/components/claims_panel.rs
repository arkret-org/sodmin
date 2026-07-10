//! Claims admin lifecycle surface used on the coauth account detail
//! page. Lists current claims and exposes a `Revoke claim` button that
//! posts to the coauth claims revoke endpoint via
//! `api::coauth::revoke_account_claim`.
//!
//! Like `did_binding_panel`, the parent passes an `on_mutated` callback
//! so it can re-fetch the account snapshot after a successful revoke.

use dioxus::prelude::*;

use crate::api::coauth::{self, CoauthAccountClaim};
use crate::components::dangerous_action_dialog::{DangerousActionDialog, confirmation_suffix};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::utils::i18n::t;
use crate::utils::net::error::HttpError;

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
            h2 { class: "text-base font-semibold", {t("claims_panel.title")} }

            if claims.is_empty() {
                p { class: "text-sm text-muted-foreground",
                    {t("claims_panel.empty")}
                }
            } else {
                ul { class: "space-y-2",
                    for claim in claims.iter() {
                        {
                            let claim_id_for_button = claim.id.clone();
                            let claim_kind_display = claim.claim_kind.clone();
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
                                        div { class: "font-medium", "{claim_kind_display}" }
                                        if !already_revoked {
                                            Button {
                                                variant: ButtonVariant::Destructive,
                                                onclick: move |_| {
                                                    pending_revoke.set(Some(claim_id_for_button.clone()));
                                                    revoke_error.set(None);
                                                },
                                                {t("common.revoke")}
                                            }
                                        } else {
                                            span { class: "text-xs text-muted-foreground", {t("claims_panel.already_revoked")} }
                                        }
                                    }
                                    div { class: "grid gap-2 text-sm md:grid-cols-3",
                                        div {
                                            span { class: "text-muted-foreground", {t("claims_panel.label_value")} }
                                            span { class: "break-all", "{value}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground", {t("claims_panel.label_state")} }
                                            span { class: "font-mono", "{state}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground", {t("claims_panel.label_source")} }
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

            DangerousActionDialog {
                open: pending_revoke.read().is_some(),
                confirmation_phrase: confirmation_suffix(pending_revoke.read().as_deref().unwrap_or(""), 4),
                title: t("claims_panel.revoke_title"),
                description: pending_revoke
                    .read()
                    .clone()
                    .map(|c| {
                        t("claims_panel.revoke_description")
                            .replace("{claim_id}", &c)
                            .replace("{account_id}", &account_id)
                    })
                    .unwrap_or_default(),
                confirm_text: if *revoke_in_flight.read() { t("claims_panel.revoking") } else { t("common.revoke") },
                cancel_text: t("common.cancel"),
                on_confirm: {
                    move |_| {
                        let claim_id = match pending_revoke.read().clone() {
                            Some(c) => c,
                            None => return,
                        };
                        spawn(async move {
                            revoke_in_flight.set(true);
                            revoke_error.set(None);
                            match coauth::revoke_account_claim(&claim_id).await {
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
        0 => t("claims_panel.error_network").replace("{message}", &e.message),
        s => t("claims_panel.error_http")
            .replace("{status}", &s.to_string())
            .replace("{message}", &e.message),
    }
}
