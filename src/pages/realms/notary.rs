//! Read-only Notary cell inspection.

use arkret_wire::{NotarySignerDescriptor, NotaryValue};
use dioxus::prelude::*;

use crate::api::seal;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::types::seal::AdminNotaryValue;
use crate::utils::i18n::t;

#[component]
pub fn NotaryPage(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let realm_id_for_fetch = realm_id.clone();
    let mut data = use_resource(move || {
        let id = realm_id_for_fetch.clone();
        async move { seal::get_notary_value(&id).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("realm_notary.title").replace("{realm_id}", &realm_id),
                description: t("realm_notary.description"),
            }

            match &*data.read() {
                Some(Ok(value)) => rsx! {
                    Card {
                        CardHeader { CardTitle { {t("realm_notary.current_value_title")} } }
                        CardContent {
                            div { class: "space-y-2 text-sm",
                                div { class: "flex items-center gap-2",
                                    Badge { variant: BadgeVariant::Secondary, "{value.kind_label()}" }
                                    span { class: "text-muted-foreground", "{value.summary()}" }
                                    if value.paused {
                                        Badge { variant: BadgeVariant::Destructive, {t("realm_notary.paused")} }
                                    }
                                }
                                {render_value_detail(value)}
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

fn render_value_detail(v: &AdminNotaryValue) -> Element {
    let n = v.notary.signers.len();
    let f = v.notary.fault_tolerance;
    let quorum = v.notary.quorum_size();
    let actor_ids = actor_ids(&v.notary.signers);
    rsx! {
        div { class: "font-mono text-xs", "f/n: {f}/{n}; quorum: {quorum}" }
        ul { class: "list-disc list-inside text-xs font-mono",
            for actor_id in actor_ids.iter() { li { "{actor_id}" } }
        }
    }
}

fn actor_ids(members: &[NotarySignerDescriptor]) -> Vec<String> {
    members
        .iter()
        .map(|member| member.actor_id.to_string())
        .collect()
}
