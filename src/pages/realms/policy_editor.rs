//! Realm policy editor
//!
//! Form to view and edit a Realm's `ck.component.realm.policy.v1`
//! components. Submit constructs a cas-register Move via
//! `POST /_soland/admin/realms/{id}/policy`. The Submit flow goes
//! through ConfirmDialog because policy mutations land permanently in
//! the Anchor frontier.

use dioxus::prelude::*;

use crate::api::realm_policy;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::realm_policy::{
    RealmDisappearingPolicy, RealmPolicy, RealmSearchPolicy, UpdateRealmPolicyRequest,
};
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
    let mut disappearing_enabled = use_signal(|| false);
    let mut disappearing_max_ttl_ms = use_signal(|| "86400000".to_string());
    let mut disappearing_allowed_triggers = use_signal(|| "on_send".to_string());
    let mut disappearing_default_grace_ms = use_signal(|| "0".to_string());
    let mut disappearing_allow_plaintext_realms = use_signal(|| false);
    let mut search_enabled_profile_refs = use_signal(String::new);
    let mut search_allowed_service_dids = use_signal(String::new);
    let mut search_data_classes = use_signal(|| "encrypted_index".to_string());
    let mut search_index_retention_ms = use_signal(String::new);
    let mut search_revocation_behavior = use_signal(|| "fail_closed".to_string());
    let mut note = use_signal(String::new);
    let mut show_confirm = use_signal(|| false);
    let mut in_flight = use_signal(|| false);
    let mut form_seeded = use_signal(|| false);

    let id_for_resource = realm_id.clone();
    let mut data = use_resource(move || {
        let id = id_for_resource.clone();
        async move { realm_policy::get_policy(&id).await }
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
        let disappearing = p.disappearing_policy.clone().unwrap_or_default();
        disappearing_enabled.set(disappearing.enabled);
        disappearing_max_ttl_ms.set(disappearing.max_ttl_ms.to_string());
        disappearing_allowed_triggers.set(join_list(&disappearing.allowed_triggers));
        disappearing_default_grace_ms.set(
            disappearing
                .default_grace_ms
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        disappearing_allow_plaintext_realms
            .set(disappearing.allow_plaintext_realms.unwrap_or(false));
        let search = p.search_policy.clone().unwrap_or_default();
        search_enabled_profile_refs.set(join_list(&search.enabled_profile_refs));
        search_allowed_service_dids.set(join_list(&search.allowed_service_dids));
        search_data_classes.set(join_list(&search.data_classes));
        search_index_retention_ms.set(
            search
                .index_retention_ms
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        search_revocation_behavior.set(search.revocation_behavior.unwrap_or_default());
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
                        div { class: "grid gap-3 md:grid-cols-2",
                            div { class: "flex items-center gap-2",
                                input {
                                    r#type: "checkbox",
                                    checked: *disappearing_enabled.read(),
                                    onchange: move |evt: Event<FormData>| disappearing_enabled.set(evt.checked()),
                                }
                                Label { "Disappearing enabled" }
                            }
                            div { class: "flex items-center gap-2",
                                input {
                                    r#type: "checkbox",
                                    checked: *disappearing_allow_plaintext_realms.read(),
                                    onchange: move |evt: Event<FormData>| disappearing_allow_plaintext_realms.set(evt.checked()),
                                }
                                Label { "Allow plaintext Realms" }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "policy-disappearing-max-ttl".to_string(),
                                    "Max TTL ms"
                                }
                                Input {
                                    value: disappearing_max_ttl_ms.read().clone(),
                                    placeholder: "86400000".to_string(),
                                    oninput: move |evt: FormEvent| disappearing_max_ttl_ms.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "policy-disappearing-triggers".to_string(),
                                    "Allowed triggers"
                                }
                                Input {
                                    value: disappearing_allowed_triggers.read().clone(),
                                    placeholder: "on_send, on_first_read, on_last_read".to_string(),
                                    oninput: move |evt: FormEvent| disappearing_allowed_triggers.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "policy-disappearing-grace".to_string(),
                                    "Default grace ms"
                                }
                                Input {
                                    value: disappearing_default_grace_ms.read().clone(),
                                    placeholder: "0".to_string(),
                                    oninput: move |evt: FormEvent| disappearing_default_grace_ms.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "policy-search-revocation".to_string(),
                                    "Search revocation"
                                }
                                Input {
                                    value: search_revocation_behavior.read().clone(),
                                    placeholder: "fail_closed / drop_stale".to_string(),
                                    oninput: move |evt: FormEvent| search_revocation_behavior.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "policy-search-profiles".to_string(),
                                    "Search profiles"
                                }
                                Input {
                                    value: search_enabled_profile_refs.read().clone(),
                                    placeholder: "ck.profile.search.client_index.v1".to_string(),
                                    oninput: move |evt: FormEvent| search_enabled_profile_refs.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "policy-search-dids".to_string(),
                                    "Search service DIDs"
                                }
                                Input {
                                    value: search_allowed_service_dids.read().clone(),
                                    placeholder: "did:web:search.example".to_string(),
                                    oninput: move |evt: FormEvent| search_allowed_service_dids.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "policy-search-data-classes".to_string(),
                                    "Search data classes"
                                }
                                Input {
                                    value: search_data_classes.read().clone(),
                                    placeholder: "encrypted_index, blind_tokens".to_string(),
                                    oninput: move |evt: FormEvent| search_data_classes.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "policy-search-retention".to_string(),
                                    "Index retention ms"
                                }
                                Input {
                                    value: search_index_retention_ms.read().clone(),
                                    placeholder: "2592000000".to_string(),
                                    oninput: move |evt: FormEvent| search_index_retention_ms.set(evt.value()),
                                }
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
                                        *disappearing_enabled.read(),
                                        &disappearing_max_ttl_ms.read(),
                                        &disappearing_allowed_triggers.read(),
                                        &disappearing_default_grace_ms.read(),
                                        *disappearing_allow_plaintext_realms.read(),
                                        &search_enabled_profile_refs.read(),
                                        &search_allowed_service_dids.read(),
                                        &search_data_classes.read(),
                                        &search_index_retention_ms.read(),
                                        &search_revocation_behavior.read(),
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
                        *disappearing_enabled.read(),
                        &disappearing_max_ttl_ms.read(),
                        &disappearing_allowed_triggers.read(),
                        &disappearing_default_grace_ms.read(),
                        *disappearing_allow_plaintext_realms.read(),
                        &search_enabled_profile_refs.read(),
                        &search_allowed_service_dids.read(),
                        &search_data_classes.read(),
                        &search_index_retention_ms.read(),
                        &search_revocation_behavior.read(),
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
                        let res = realm_policy::update_policy(&id, &req).await;
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
    disappearing_enabled: bool,
    disappearing_max_ttl_ms: &str,
    disappearing_allowed_triggers: &str,
    disappearing_default_grace_ms: &str,
    disappearing_allow_plaintext_realms: bool,
    search_enabled_profile_refs: &str,
    search_allowed_service_dids: &str,
    search_data_classes: &str,
    search_index_retention_ms: &str,
    search_revocation_behavior: &str,
) -> RealmPolicy {
    RealmPolicy {
        history_visibility: history_visibility.trim().to_string(),
        join_rule: join_rule.trim().to_string(),
        guest_access: guest_access.trim().to_string(),
        federate,
        encryption_algorithm: encryption_algorithm.trim().to_string(),
        disappearing_policy: Some(RealmDisappearingPolicy {
            enabled: disappearing_enabled,
            max_ttl_ms: disappearing_max_ttl_ms.trim().parse::<u64>().unwrap_or(0),
            allowed_triggers: parse_list(disappearing_allowed_triggers),
            default_grace_ms: parse_optional_u64(disappearing_default_grace_ms),
            allow_plaintext_realms: Some(disappearing_allow_plaintext_realms),
        }),
        search_policy: Some(RealmSearchPolicy {
            enabled_profile_refs: parse_list(search_enabled_profile_refs),
            allowed_service_dids: parse_list(search_allowed_service_dids),
            data_classes: parse_list(search_data_classes),
            index_retention_ms: parse_optional_u64(search_index_retention_ms),
            revocation_behavior: {
                let value = search_revocation_behavior.trim();
                if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                }
            },
        }),
    }
}

fn join_list(values: &[String]) -> String {
    values.join(", ")
}

fn parse_list(value: &str) -> Vec<String> {
    value
        .split([',', '\n'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_optional_u64(value: &str) -> Option<u64> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        value.parse::<u64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_policy_trims_whitespace() {
        let p = current_policy(
            "  joined ",
            "invite ",
            " forbidden",
            true,
            "  ",
            false,
            "86400000",
            "on_send",
            "0",
            false,
            "",
            "",
            "encrypted_index",
            "",
            "fail_closed",
        );
        assert_eq!(p.history_visibility, "joined");
        assert_eq!(p.join_rule, "invite");
        assert_eq!(p.guest_access, "forbidden");
        assert_eq!(p.encryption_algorithm, "");
        assert!(p.federate);
    }

    #[test]
    fn current_policy_validates_after_trim() {
        let p = current_policy(
            "  joined ",
            "invite",
            "",
            true,
            "",
            true,
            "3600000",
            "on_send, on_first_read",
            "0",
            false,
            "ck.profile.search.client_index.v1",
            "did:web:search.example",
            "encrypted_index",
            "",
            "fail_closed",
        );
        assert!(p.validate().is_ok());
    }

    #[test]
    fn current_policy_invalid_guest_access_caught() {
        let p = current_policy(
            "joined",
            "invite",
            "garbage",
            false,
            "",
            false,
            "86400000",
            "on_send",
            "",
            false,
            "",
            "",
            "encrypted_index",
            "",
            "",
        );
        assert!(p.validate().is_err());
    }

    #[test]
    fn current_policy_uses_spec_policy_field_names() {
        let p = current_policy(
            "joined",
            "invite",
            "",
            true,
            "",
            true,
            "3600000",
            "on_send, on_last_read",
            "1000",
            false,
            "ck.profile.search.client_index.v1, ck.profile.search.blind_index.v1",
            "did:web:search.example",
            "encrypted_index, blind_tokens",
            "2592000000",
            "drop_stale",
        );
        let value = serde_json::to_value(&p).unwrap();
        assert_eq!(value["disappearing_policy"]["max_ttl_ms"], 3_600_000);
        assert_eq!(
            value["disappearing_policy"]["allowed_triggers"],
            serde_json::json!(["on_send", "on_last_read"])
        );
        assert_eq!(value["disappearing_policy"]["default_grace_ms"], 1_000);
        assert_eq!(
            value["search_policy"]["enabled_profile_refs"],
            serde_json::json!([
                "ck.profile.search.client_index.v1",
                "ck.profile.search.blind_index.v1"
            ])
        );
        assert_eq!(
            value["search_policy"]["allowed_service_dids"],
            serde_json::json!(["did:web:search.example"])
        );
        assert_eq!(
            value["search_policy"]["data_classes"],
            serde_json::json!(["encrypted_index", "blind_tokens"])
        );
        assert_eq!(
            value["search_policy"]["index_retention_ms"],
            2_592_000_000u64
        );
        assert_eq!(value["search_policy"]["revocation_behavior"], "drop_stale");
    }
}
