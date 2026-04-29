use dioxus::prelude::*;

use crate::api::server;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::utils::i18n::t;

#[component]
pub fn ServerStatus() -> Element {
    let mut info_data = use_resource(|| async { server::get_server_info().await.ok() });
    let mut status_data = use_resource(|| async { server::get_server_status().await.ok() });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("server_status.title"),
                description: t("server_status.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        info_data.restart();
                        status_data.restart();
                    },
                    {t("common.refresh")}
                }
            }

            match &*info_data.read() {
                Some(Some(info)) => {
                    let version = info.server_version.clone();
                    let protocol = info.protocol_version.clone().unwrap_or_else(|| "-".to_string());
                    let server_name = info.server_name.clone().unwrap_or_else(|| "-".to_string());
                    let uptime = info.uptime.map(|u| format!("{u}s")).unwrap_or_else(|| "-".to_string());

                    rsx! {
                        Card {
                            CardHeader { CardTitle { {t("server_status.server_info")} } }
                            CardContent {
                                div { class: "grid gap-4 md:grid-cols-4",
                                    InfoCell { label: t("server_status.version"), value: version }
                                    InfoCell { label: t("server_status.protocol_version"), value: protocol }
                                    InfoCell { label: t("server_status.server_name"), value: server_name }
                                    InfoCell { label: t("server_status.uptime"), value: uptime }
                                }
                            }
                        }
                    }
                },
                _ => rsx! {},
            }

            match &*status_data.read() {
                Some(Some(status)) => rsx! {
                    div { class: "flex items-center gap-4 mb-4",
                        if status.ok {
                            Badge { variant: BadgeVariant::Success, class: "text-base px-4 py-1".to_string(), "Healthy" }
                        } else {
                            Badge { variant: BadgeVariant::Destructive, class: "text-base px-4 py-1".to_string(), "Issues Detected" }
                        }
                    }

                    div { class: "grid gap-4 md:grid-cols-2 lg:grid-cols-3",
                        for component in status.results.iter() {
                            Card {
                                CardContent { class: "p-4".to_string(),
                                    div { class: "flex items-center justify-between",
                                        div { class: "space-y-1",
                                            p { class: "font-medium",
                                                {component.label.as_deref().unwrap_or("Unknown")}
                                            }
                                            if !component.ok {
                                                if let Some(ref reason) = component.reason {
                                                    p { class: "text-xs text-destructive mt-1", "{reason}" }
                                                }
                                            }
                                        }
                                        if component.ok {
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
                Some(None) => rsx! {
                    Card {
                        CardContent { class: "p-8 text-center".to_string(),
                            p { class: "text-muted-foreground", {t("server_status.unable")} }
                        }
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}

#[component]
fn InfoCell(label: String, value: String) -> Element {
    rsx! {
        div {
            p { class: "text-sm text-muted-foreground", "{label}" }
            p { class: "text-lg font-semibold", "{value}" }
        }
    }
}
