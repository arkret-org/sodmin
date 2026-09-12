//! Read-only view of the unique confirmed Seal head for one Realm.

use dioxus::prelude::*;

use crate::api::seal;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::card::*;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::utils::i18n::t;

#[component]
pub fn SealChainPage(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let realm_id_for_fetch = realm_id.clone();
    let mut data = use_resource(move || {
        let id = realm_id_for_fetch.clone();
        async move { seal::get_seal_chain(&id).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("realm_seal_chain.title").replace("{realm_id}", &realm_id),
                description: t("realm_seal_chain.description"),
            }

            match &*data.read() {
                Some(Ok(snapshot)) => match &snapshot.head {
                    None => rsx! {
                        EmptyState {
                            icon_name: "shield".to_string(),
                            title: t("realm_seal_chain.empty_title"),
                            description: t("realm_seal_chain.empty_description"),
                        }
                    },
                    Some(head) => {
                        let state_root = head
                            .state_root
                            .as_deref()
                            .or(snapshot.state_root.as_deref())
                            .unwrap_or("-");
                        let created_at = head.created_at.as_deref().unwrap_or("-");
                        let signer = &head.signer;
                        let covered_events = snapshot.covered_event_digests.join(", ");
                        rsx! {
                            Card {
                                CardHeader { CardTitle { {t("realm_seal_chain.card_head_title")} } }
                                CardContent {
                                    div { class: "space-y-2 text-sm",
                                        div { span { class: "text-muted-foreground mr-2", {t("realm_seal_chain.seal_id_label")} } span { class: "font-mono text-xs", "{head.seal_id}" } }
                                        div { span { class: "text-muted-foreground mr-2", {t("realm_seal_chain.state_root_label")} } span { class: "font-mono text-xs", "{state_root}" } }
                                        div { span { class: "text-muted-foreground mr-2", {t("realm_seal_chain.control_events_label")} } span { "{head.control_event_count}" } }
                                        div { span { class: "text-muted-foreground mr-2", {t("realm_seal_chain.created_label")} } span { "{created_at}" } }
                                        div { span { class: "text-muted-foreground mr-2", {t("realm_seal_chain.signer_label")} } span { class: "font-mono text-xs", "{signer}" } }
                                        div { span { class: "text-muted-foreground mr-2", {t("realm_seal_chain.covered_events_label")} } span { class: "font-mono text-xs", "{covered_events}" } }
                                    }
                                }
                            }
                        }
                    }
                },
                Some(Err(error)) => rsx! {
                    ErrorBanner {
                        message: error.message.clone(),
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
