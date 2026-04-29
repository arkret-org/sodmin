use dioxus::prelude::*;

use crate::api::federation;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::utils::i18n::t;

#[component]
pub fn FederationShow(domain: String) -> Element {
    let mut reset_loading = use_signal(|| false);

    let domain_for_resource = domain.clone();
    let mut peer_data = use_resource(move || {
        let d = domain_for_resource.clone();
        async move { federation::get_federation_peer(&d).await }
    });

    rsx! {
        div { class: "space-y-6",
            Breadcrumbs {
                items: vec![
                    BreadcrumbItem { label: t("federation.title"), route: Some(Route::FederationList {}) },
                    BreadcrumbItem { label: domain, route: None },
                ],
            }

            match &*peer_data.read() {
                Some(Ok(peer)) => {
                    let domain = peer.domain.clone();
                    let domain_for_reset = domain.clone();
                    let status = peer.status.clone().unwrap_or_else(|| "-".to_string());
                    let trust_level = peer.trust_level.clone().unwrap_or_else(|| "-".to_string());
                    let last_txn = peer.last_successful_txn.clone().unwrap_or_else(|| "-".to_string());
                    let last_error = peer.last_error.clone().unwrap_or_else(|| "-".to_string());
                    let direction = peer.direction.clone().unwrap_or_else(|| "-".to_string());
                    let is_reset_loading = *reset_loading.read();

                    rsx! {
                        PageHeader {
                            title: domain.clone(),
                            description: t("federation.peer_details"),
                            Button {
                                variant: ButtonVariant::Outline,
                                disabled: is_reset_loading,
                                onclick: move |_| {
                                    let d = domain_for_reset.clone();
                                    reset_loading.set(true);
                                    spawn(async move {
                                        match federation::reset_federation_connection(&d).await {
                                            Ok(_) => {
                                                show_toast("Connection reset", ToastVariant::Success);
                                                peer_data.restart();
                                            }
                                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                        }
                                        reset_loading.set(false);
                                    });
                                },
                                {t("federation.reset_connection")}
                            }
                        }

                        Card {
                            CardHeader { CardTitle { {t("federation.peer_info")} } }
                            CardContent {
                                div { class: "space-y-4",
                                    InfoRow { label: t("federation.domain"), value: domain }
                                    InfoRow { label: t("federation.status"), value: status }
                                    InfoRow { label: t("federation.trust_level"), value: trust_level }
                                    InfoRow { label: t("federation.direction"), value: direction }
                                    InfoRow { label: t("federation.last_successful_txn"), value: last_txn }
                                    InfoRow { label: t("federation.last_error"), value: last_error }
                                }
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        on_retry: move |_| peer_data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}

#[component]
fn InfoRow(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between py-2",
            span { class: "text-sm font-medium text-muted-foreground", "{label}" }
            span { class: "text-sm max-w-[60%] text-right break-all", "{value}" }
        }
    }
}
