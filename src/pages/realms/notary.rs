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
                                if let Some(ms) = value.revocation_freshness_window_ms {
                                    p { class: "text-muted-foreground",
                                        {format!("revocation_freshness_window_ms: {}", ms)}
                                    }
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

fn render_value_detail(v: &AdminNotaryValue) -> Element {
    match &v.notary {
        NotaryValue::SingleSigner { signer, .. } => {
            let did = signer.actor_id.to_string();
            rsx! { div { class: "font-mono text-xs", "did: {did}" } }
        }
        NotaryValue::Threshold {
            members, threshold, ..
        } => {
            let k = *threshold;
            let n = members.len();
            let dids = actor_ids(members);
            rsx! {
                div { class: "font-mono text-xs", "k/n: {k}/{n}" }
                ul { class: "list-disc list-inside text-xs font-mono",
                    for did in dids.iter() { li { "{did}" } }
                }
            }
        }
        NotaryValue::OpenSet { members } => {
            let dids = actor_ids(members);
            rsx! {
                ul { class: "list-disc list-inside text-xs font-mono",
                    for did in dids.iter() { li { "{did}" } }
                }
            }
        }
        NotaryValue::Mixed {
            signer,
            recovery_members,
            ..
        } => {
            let primary = signer.actor_id.to_string();
            let recovery = actor_ids(recovery_members);
            rsx! {
                div { class: "font-mono text-xs", "primary: {primary}" }
                div { class: "text-xs text-muted-foreground", "recovery:" }
                ul { class: "list-disc list-inside text-xs font-mono",
                    for did in recovery.iter() { li { "{did}" } }
                }
            }
        }
    }
}

fn actor_ids(members: &[NotarySignerDescriptor]) -> Vec<String> {
    members
        .iter()
        .map(|member| member.actor_id.to_string())
        .collect()
}
