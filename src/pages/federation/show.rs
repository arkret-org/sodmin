use dioxus::prelude::*;

use crate::api::federation;
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::info_row::InfoRow;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::router::Route;
use crate::utils::i18n::t;

#[component]
pub fn FederationShow(operation_id: String) -> Element {
    let operation_id_for_resource = operation_id.clone();
    let mut operation_data = use_resource(move || {
        let id = operation_id_for_resource.clone();
        async move { federation::get_federation_operation(&id).await }
    });

    rsx! {
        div { class: "space-y-6",
            Breadcrumbs {
                items: vec![
                    BreadcrumbItem { label: t("federation.title"), route: Some(Route::FederationList {}) },
                    BreadcrumbItem { label: operation_id, route: None },
                ],
            }

            match &*operation_data.read() {
                Some(Ok(op)) => {
                    let operation_id = op.operation_id.clone();
                    let realm_id = op.realm_id.clone().unwrap_or_else(|| "-".to_string());
                    let operation_type = op.operation_type.clone().unwrap_or_else(|| "-".to_string());
                    let canonical_kind = op.canonical_kind.clone().unwrap_or_else(|| "-".to_string());
                    let strand_id = op.strand_id.clone().unwrap_or_else(|| "-".to_string());
                    let track = op.track.clone().unwrap_or_else(|| "-".to_string());
                    let digest = op.digest.clone().unwrap_or_else(|| "-".to_string());
                    let created_at = op.created_at.clone().unwrap_or_else(|| "-".to_string());
                    rsx! {
                        PageHeader {
                            title: operation_id.clone(),
                            description: t("federation.operation_details"),
                        }

                        Card {
                            CardHeader { CardTitle { {t("federation.operation_info")} } }
                            CardContent {
                                div { class: "space-y-4",
                                    InfoRow { label: t("federation.operation_id"), value: operation_id }
                                    InfoRow { label: t("federation.realm_id"), value: realm_id }
                                    InfoRow { label: t("federation.operation_type"), value: operation_type }
                                    InfoRow { label: t("federation.canonical_kind"), value: canonical_kind }
                                    InfoRow { label: t("federation.strand_id"), value: strand_id }
                                    InfoRow { label: t("federation.track"), value: track }
                                    InfoRow { label: t("federation.digest"), value: digest }
                                    InfoRow { label: t("federation.created_at"), value: created_at }
                                }
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        on_retry: move |_| operation_data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
