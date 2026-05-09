//! DID-binding admin surface used on the coauth account detail page.
//!
//! Renders the current set of `managed_dids`, plus an inline `Add binding`
//! form (did + control_proof) and a `Remove binding` confirmation modal.
//! Submission flows through the dedicated API helpers in
//! `api::coauth::add_account_did_binding` /
//! `api::coauth::remove_account_did_binding`.
//!
//! `on_mutated` is invoked on success so the parent can `data.restart()`
//! its snapshot resource and pull the fresh list back from the server.

use dioxus::prelude::*;

use crate::api::coauth::{
    self, CoauthManagedDidBinding,
};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::input::{Input, Label};
use crate::utils::error::HttpError;

#[component]
pub fn DidBindingPanel(
    account_id: String,
    bindings: Vec<CoauthManagedDidBinding>,
    on_mutated: EventHandler<()>,
) -> Element {
    let mut new_did = use_signal(String::new);
    let mut new_control_proof = use_signal(String::new);
    let mut submit_in_flight = use_signal(|| false);
    let mut submit_error = use_signal(|| None::<String>);

    // `Some(did)` when the user has clicked "Remove" on a row but not yet
    // confirmed the modal.
    let mut pending_remove = use_signal(|| None::<String>);
    let mut remove_in_flight = use_signal(|| false);
    let mut remove_error = use_signal(|| None::<String>);

    rsx! {
        div { class: "rounded-lg border p-4 space-y-3",
            h2 { class: "text-base font-semibold", "Managed DID Bindings" }

            if bindings.is_empty() {
                p { class: "text-sm text-muted-foreground",
                    "No DID bindings are currently returned for this account."
                }
            } else {
                ul { class: "space-y-2",
                    for binding in bindings.iter() {
                        {
                            let did_for_button = binding.did.clone();
                            let did_display = binding.did.clone();
                            let method = binding.method.clone().unwrap_or_else(|| "-".to_string());
                            let state = binding.state.clone().unwrap_or_else(|| "-".to_string());
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
                                            "Remove"
                                        }
                                    }
                                    div { class: "grid gap-2 text-sm md:grid-cols-3",
                                        div {
                                            span { class: "text-muted-foreground", "Method: " }
                                            span { "{method}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground", "State: " }
                                            span { class: "font-mono", "{state}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground", "Last Verified: " }
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
                h3 { class: "text-sm font-semibold", "Add DID Binding" }
                p { class: "text-xs text-muted-foreground",
                    "Both fields are required. The control_proof is an opaque blob the backend forwards to the DID resolver."
                }
                div { class: "space-y-2",
                    Label { r#for: "new-did".to_string(), "DID" }
                    Input {
                        r#type: "text".to_string(),
                        placeholder: "did:web:example.org:account:alice".to_string(),
                        value: new_did.read().clone(),
                        oninput: move |evt: FormEvent| new_did.set(evt.value()),
                    }
                }
                div { class: "space-y-2",
                    Label { r#for: "new-control-proof".to_string(), "Control Proof" }
                    Input {
                        r#type: "text".to_string(),
                        placeholder: "base64url-encoded signed challenge".to_string(),
                        value: new_control_proof.read().clone(),
                        oninput: move |evt: FormEvent| new_control_proof.set(evt.value()),
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
                        || new_control_proof.read().trim().is_empty(),
                    onclick: {
                        let account_id = account_id.clone();
                        move |_| {
                            let account_id = account_id.clone();
                            let did = new_did.read().trim().to_string();
                            let proof = new_control_proof.read().trim().to_string();
                            spawn(async move {
                                submit_in_flight.set(true);
                                submit_error.set(None);
                                match coauth::add_account_did_binding(&account_id, &did, &proof).await {
                                    Ok(()) => {
                                        new_did.set(String::new());
                                        new_control_proof.set(String::new());
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
                    if *submit_in_flight.read() { "Submitting..." } else { "Add binding" }
                }
            }

            if let Some(err) = remove_error.read().clone() {
                div { class: "rounded-md bg-destructive/10 p-2 text-sm text-destructive",
                    "{err}"
                }
            }

            ConfirmDialog {
                open: pending_remove.read().is_some(),
                title: "Remove DID binding?".to_string(),
                description: pending_remove
                    .read()
                    .clone()
                    .map(|d| format!(
                        "This will detach {} from the account. The DID itself is not deleted.",
                        d
                    ))
                    .unwrap_or_default(),
                confirm_text: if *remove_in_flight.read() { "Removing...".to_string() } else { "Remove".to_string() },
                cancel_text: "Cancel".to_string(),
                destructive: true,
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
        0 => format!("Network error: {}", e.message),
        s => format!("HTTP {}: {}", s, e.message),
    }
}
