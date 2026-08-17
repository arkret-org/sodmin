//! DID-binding admin surface used on the coauth account detail page.
//!
//! Renders the current set of `managed_dids`, plus an inline `Add binding`
//! form (did + control_proof) and a `Remove binding` confirmation modal.
//! Submission strands through the dedicated API helpers in
//! `api::coauth::add_account_did_binding` /
//! `api::coauth::remove_account_did_binding`.
//!
//! `on_mutated` is invoked on success so the parent can `data.restart()`
//! its snapshot resource and pull the fresh list back from the server.

use dioxus::prelude::*;

use crate::api::coauth::{self, CoauthDidBindingKind, CoauthManagedDidBinding};
use crate::components::dangerous_action_dialog::{DangerousActionDialog, confirmation_suffix};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::Label;
use crate::components::validated_input::{ValidatedInput, ValidationKind};
use crate::utils::i18n::t;
use crate::utils::net::error::HttpError;

#[component]
pub fn DidBindingPanel(
    account_id: String,
    bindings: Vec<CoauthManagedDidBinding>,
    on_mutated: EventHandler<()>,
) -> Element {
    let mut new_did = use_signal(String::new);
    let mut new_kind = use_signal(|| "primary".to_string());
    let mut new_control_proof_jws = use_signal(String::new);
    let mut new_control_proof_nonce = use_signal(String::new);
    let mut submit_in_flight = use_signal(|| false);
    let mut submit_error = use_signal(|| None::<String>);

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

            div { class: "rounded-md border p-3 space-y-3",
                h3 { class: "text-sm font-semibold", {t("did_binding_panel.add_title")} }
                p { class: "text-xs text-muted-foreground",
                    {t("did_binding_panel.add_hint")}
                }
                div { class: "space-y-2",
                    Label { r#for: "new-did".to_string(), {t("did_binding_panel.field_did")} }
                    // ValidatedInput enforces the DID grammar with
                    // validation identical to `arkret_identifiers::is_did`.
                    ValidatedInput {
                        kind: ValidationKind::Did,
                        placeholder: "did:webvh:example.org:account:alice".to_string(),
                        value: new_did.read().clone(),
                        oninput: move |evt: FormEvent| new_did.set(evt.value()),
                    }
                }
                div { class: "space-y-2",
                    Label { r#for: "new-kind".to_string(), {t("did_binding_panel.field_kind")} }
                    select {
                        class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                        value: new_kind.read().clone(),
                        onchange: move |evt: Event<FormData>| new_kind.set(evt.value()),
                        option { value: "primary", {t("did_binding_panel.kind_primary")} }
                        option { value: "recovery", {t("did_binding_panel.kind_recovery")} }
                        option { value: "pairwise", {t("did_binding_panel.kind_pairwise")} }
                    }
                }
                div { class: "space-y-2",
                    Label { r#for: "new-control-proof-jws".to_string(), {t("did_binding_panel.field_control_proof_jws")} }
                    ValidatedInput {
                        kind: ValidationKind::Required,
                        placeholder: t("did_binding_panel.jws_placeholder"),
                        value: new_control_proof_jws.read().clone(),
                        oninput: move |evt: FormEvent| new_control_proof_jws.set(evt.value()),
                    }
                }
                div { class: "space-y-2",
                    Label { r#for: "new-control-proof-nonce".to_string(), {t("did_binding_panel.field_control_proof_nonce")} }
                    ValidatedInput {
                        kind: ValidationKind::Required,
                        placeholder: t("did_binding_panel.nonce_placeholder"),
                        value: new_control_proof_nonce.read().clone(),
                        oninput: move |evt: FormEvent| new_control_proof_nonce.set(evt.value()),
                    }
                }

                if let Some(err) = submit_error.read().clone() {
                    div { class: "rounded-md bg-destructive/10 p-2 text-sm text-destructive",
                        "{err}"
                    }
                }

                Button {
                    variant: ButtonVariant::Default,
                    disabled: *submit_in_flight.read()
                        || new_did.read().trim().is_empty()
                        || new_control_proof_jws.read().trim().is_empty()
                        || new_control_proof_nonce.read().trim().is_empty(),
                    onclick: {
                        let account_id = account_id.clone();
                        move |_| {
                            let account_id = account_id.clone();
                            let did = new_did.read().trim().to_string();
                            let kind = did_binding_kind_from_form(&new_kind.read());
                            let proof_jws = new_control_proof_jws.read().trim().to_string();
                            let proof_nonce = new_control_proof_nonce.read().trim().to_string();
                            spawn(async move {
                                submit_in_flight.set(true);
                                submit_error.set(None);
                                match coauth::add_account_did_binding(
                                    &account_id,
                                    &did,
                                    kind,
                                    &proof_jws,
                                    &proof_nonce,
                                )
                                .await
                                {
                                    Ok(()) => {
                                        new_did.set(String::new());
                                        new_kind.set("primary".to_string());
                                        new_control_proof_jws.set(String::new());
                                        new_control_proof_nonce.set(String::new());
                                        submit_in_flight.set(false);
                                        on_mutated.call(());
                                    }
                                    Err(e) => {
                                        submit_in_flight.set(false);
                                        submit_error.set(Some(format_err(&e)));
                                    }
                                }
                            });
                        }
                    },
                    if *submit_in_flight.read() {
                        {t("did_binding_panel.submitting")}
                    } else {
                        {t("did_binding_panel.add_binding")}
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

fn did_binding_kind_from_form(value: &str) -> CoauthDidBindingKind {
    match value {
        "recovery" => CoauthDidBindingKind::Recovery,
        "pairwise" => CoauthDidBindingKind::Pairwise,
        _ => CoauthDidBindingKind::Primary,
    }
}
