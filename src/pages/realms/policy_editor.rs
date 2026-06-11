use dioxus::prelude::*;

use crate::api::realm_policy;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::utils::i18n::t;

#[component]
pub fn PolicyEditorPage(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let id_for_resource = realm_id.clone();
    let mut data = use_resource(move || {
        let id = id_for_resource.clone();
        async move { realm_policy::get_policy(&id).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("policy_editor.title"),
                description: t("policy_editor.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            p { class: "text-xs text-muted-foreground",
                {format!("Realm: {realm_id}")}
            }

            match &*data.read() {
                Some(Ok(policy)) => {
                    let disappearing = policy.disappearing_policy.as_ref();
                    let search = policy.search_policy.as_ref();
                    rsx! {
                        Card {
                            CardHeader { CardTitle { "Realm policy projection" } }
                            CardContent {
                                div { class: "space-y-3 text-sm",
                                    {field_row("history_visibility", &policy.history_visibility)}
                                    {field_row("join_rule", &policy.join_rule)}
                                    {field_row("guest_access", &policy.guest_access)}
                                    {field_row("federate", if policy.federate { "true" } else { "false" })}
                                    {field_row("encryption_algorithm", &policy.encryption_algorithm)}
                                    {field_row(
                                        "disappearing_policy",
                                        disappearing
                                            .map(|value| if value.enabled { "enabled" } else { "disabled" })
                                            .unwrap_or("-"),
                                    )}
                                    {field_row(
                                        "search_policy",
                                        search
                                            .map(|value| if value.enabled_profile_refs.is_empty() { "disabled" } else { "enabled" })
                                            .unwrap_or("-"),
                                    )}
                                }
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
        }
    }
}

fn field_row(label: &'static str, value: &str) -> Element {
    rsx! {
        div { class: "flex justify-between gap-4 border-b border-border/50 py-1.5 last:border-0",
            span { class: "text-muted-foreground", "{label}" }
            span { class: "font-mono text-xs text-right break-all", "{value}" }
        }
    }
}
