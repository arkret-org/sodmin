use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::utils::i18n::t;

#[component]
pub fn ConnectorHealthPage() -> Element {
    let mut data = use_resource(|| async { coauth::get_connector_health().await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth.connector_health.title"),
                description: t("coauth.connector_health.subtitle"),
            }

            match &*data.read() {
                Some(Ok(connectors)) => rsx! {
                    div { class: "grid gap-4 md:grid-cols-2 lg:grid-cols-3",
                        for connector in connectors.iter() {
                            Card {
                                CardContent { class: "p-4".to_string(),
                                    div { class: "flex items-center justify-between",
                                        div { class: "space-y-1",
                                            p { class: "font-medium", "{connector.provider}" }
                                            p { class: "text-xs text-muted-foreground", "{connector.account_id}" }
                                            if !connector.is_healthy() {
                                                if let Some(ref err) = connector.error {
                                                    p { class: "text-xs text-destructive mt-1", "{err}" }
                                                }
                                            }
                                        }
                                        if connector.is_healthy() {
                                            Badge { variant: BadgeVariant::Success, "OK" }
                                        } else {
                                            Badge { variant: BadgeVariant::Destructive, "Error" }
                                        }
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
