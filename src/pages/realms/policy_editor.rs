//! Realm policy editor
//!
//! Form to view and edit a Realm's `ck.component.realm.policy.v1`
//! components. Submit constructs a cas-register Move via
//! `POST /_soland/admin/realms/{id}/policy`. The Submit flow goes
//! through ConfirmDialog because policy mutations land permanently in
//! the Anchor frontier.

use dioxus::prelude::*;

use crate::api::realm_policy_admin;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::realm_policy::{RealmPolicy, UpdateRealmPolicyRequest};
use crate::utils::i18n::t;
use crate::utils::net::error::format_optional_endpoint_error;

#[component]
pub fn PolicyEditorPage(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let mut history_visibility = use_signal(String::new);
    let mut join_rule = use_signal(String::new);
    let mut guest_access = use_signal(String::new);
    let mut federate = use_signal(|| true);
    let mut encryption_algorithm = use_signal(String::new);
    let mut note = use_signal(String::new);
    let mut show_confirm = use_signal(|| false);
    let mut in_flight = use_signal(|| false);
    let mut form_seeded = use_signal(|| false);

    let id_for_resource = realm_id.clone();
    let mut data = use_resource(move || {
        let id = id_for_resource.clone();
        async move { realm_policy_admin::get_policy(&id).await }
    });

    // Seed the form once from the first successful read. Subsequent
    // refreshes leave the operator's edits in place.
    if !*form_seeded.read()
        && let Some(Ok(p)) = data.read().as_ref()
    {
        history_visibility.set(p.history_visibility.clone());
        join_rule.set(p.join_rule.clone());
        guest_access.set(p.guest_access.clone());
        federate.set(p.federate);
        encryption_algorithm.set(p.encryption_algorithm.clone());
        form_seeded.set(true);
    }

    let id_for_submit = realm_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("policy_editor.title"),
                description: t("policy_editor.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        form_seeded.set(false);
                        data.restart();
                    },
                    {t("common.refresh")}
                }
            }

            p { class: "text-xs text-muted-foreground",
                {format!("Realm: {realm_id}")}
            }

            match &*data.read() {
                Some(Ok(_)) => rsx! {
                    div { class: "space-y-3 rounded-md border p-4",
                        div { class: "space-y-1",
                            Label { r#for: "policy-history-visibility".to_string(),
                                {t("policy_editor.history_visibility")}
                            }
                            Input {
                                value: history_visibility.read().clone(),
                                placeholder: "joined / invited / shared / world_readable".to_string(),
                                oninput: move |evt: FormEvent| history_visibility.set(evt.value()),
                            }
                        }
                        div { class: "space-y-1",
                            Label { r#for: "policy-join-rule".to_string(),
                                {t("policy_editor.join_rule")}
                            }
                            Input {
                                value: join_rule.read().clone(),
                                placeholder: "public / invite / restricted / knock".to_string(),
                                oninput: move |evt: FormEvent| join_rule.set(evt.value()),
                            }
                        }
                        div { class: "space-y-1",
                            Label { r#for: "policy-guest-access".to_string(),
                                {t("policy_editor.guest_access")}
                            }
                            Input {
                                value: guest_access.read().clone(),
                                placeholder: "can_join / forbidden".to_string(),
                                oninput: move |evt: FormEvent| guest_access.set(evt.value()),
                            }
                        }
                        div { class: "flex items-center gap-2",
                            input {
                                r#type: "checkbox",
                                checked: *federate.read(),
                                onchange: move |evt: Event<FormData>| federate.set(evt.checked()),
                            }
                            Label { {t("policy_editor.federate")} }
                        }
                        div { class: "space-y-1",
                            Label { r#for: "policy-encryption".to_string(),
                                {t("policy_editor.encryption_algorithm")}
                            }
                            Input {
                                value: encryption_algorithm.read().clone(),
                                placeholder: "(blank = no E2EE)".to_string(),
                                oninput: move |evt: FormEvent| encryption_algorithm.set(evt.value()),
                            }
                        }
                        div { class: "space-y-1",
                            Label { r#for: "policy-note".to_string(),
                                {t("policy_editor.note")}
                            }
                            Input {
                                value: note.read().clone(),
                                placeholder: "Optional commit note (audit-logged)".to_string(),
                                oninput: move |evt: FormEvent| note.set(evt.value()),
                            }
                        }
                        div { class: "flex justify-end",
                            Button {
                                variant: ButtonVariant::Default,
                                disabled: *in_flight.read(),
                                onclick: move |_| {
                                    let p = current_policy(
                                        &history_visibility.read(),
                                        &join_rule.read(),
                                        &guest_access.read(),
                                        *federate.read(),
                                        &encryption_algorithm.read(),
                                    );
                                    match p.validate() {
                                        Ok(()) => show_confirm.set(true),
                                        Err(msg) => show_toast(&msg, ToastVariant::Error),
                                    }
                                },
                                {t("policy_editor.submit")}
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }

            ConfirmDialog {
                open: *show_confirm.read(),
                title: t("policy_editor.confirm_title"),
                description: t("policy_editor.confirm_body"),
                confirm_text: t("policy_editor.confirm_btn"),
                cancel_text: t("common.cancel"),
                destructive: true,
                on_cancel: move |_| show_confirm.set(false),
                on_confirm: move |_| {
                    show_confirm.set(false);
                    let policy = current_policy(
                        &history_visibility.read(),
                        &join_rule.read(),
                        &guest_access.read(),
                        *federate.read(),
                        &encryption_algorithm.read(),
                    );
                    let req = UpdateRealmPolicyRequest {
                        policy,
                        note: {
                            let n = note.read().clone();
                            if n.trim().is_empty() { None } else { Some(n) }
                        },
                    };
                    let id = id_for_submit.clone();
                    in_flight.set(true);
                    spawn(async move {
                        let res = realm_policy_admin::update_policy(&id, &req).await;
                        match res {
                            Ok(_) => show_toast(
                                "Realm policy updated.",
                                ToastVariant::Success,
                            ),
                            Err(e) => {
                                let msg = format_optional_endpoint_error(
                                    "realm policy update",
                                    &e,
                                );
                                show_toast(&msg, ToastVariant::Error);
                            }
                        }
                        in_flight.set(false);
                        data.restart();
                    });
                },
            }
        }
    }
}

/// Pure helper — gather the current form values into a typed
/// `RealmPolicy`. Trims each text field so accidental whitespace
/// doesn't slip into the cas-register Move.
pub(crate) fn current_policy(
    history_visibility: &str,
    join_rule: &str,
    guest_access: &str,
    federate: bool,
    encryption_algorithm: &str,
) -> RealmPolicy {
    RealmPolicy {
        history_visibility: history_visibility.trim().to_string(),
        join_rule: join_rule.trim().to_string(),
        guest_access: guest_access.trim().to_string(),
        federate,
        encryption_algorithm: encryption_algorithm.trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_policy_trims_whitespace() {
        let p = current_policy("  joined ", "invite ", " forbidden", true, "  ");
        assert_eq!(p.history_visibility, "joined");
        assert_eq!(p.join_rule, "invite");
        assert_eq!(p.guest_access, "forbidden");
        assert_eq!(p.encryption_algorithm, "");
        assert!(p.federate);
    }

    #[test]
    fn current_policy_validates_after_trim() {
        let p = current_policy("  joined ", "invite", "", true, "");
        assert!(p.validate().is_ok());
    }

    #[test]
    fn current_policy_invalid_guest_access_caught() {
        let p = current_policy("joined", "invite", "garbage", false, "");
        assert!(p.validate().is_err());
    }
}
