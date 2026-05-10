use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::utils::i18n::t;

#[component]
pub fn NotificationChannelsPage() -> Element {
    let mut data = use_resource(|| async { coauth::list_notification_channels().await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth.notification_channels.title"),
                description: t("coauth.notification_channels.subtitle"),
            }

            match &*data.read() {
                Some(Ok(channels)) => rsx! {
                    div { class: "grid gap-4 md:grid-cols-2 lg:grid-cols-3",
                        for channel in channels.iter() {
                            Card {
                                CardContent { class: "p-4".to_string(),
                                    div { class: "flex items-center justify-between",
                                        div { class: "space-y-1",
                                            p { class: "font-medium", "{channel.channel}" }
                                            if channel.configured {
                                                p { class: "text-xs text-muted-foreground",
                                                    {t("coauth.notification_channels.configured")}
                                                }
                                            } else {
                                                p { class: "text-xs text-muted-foreground",
                                                    {t("coauth.notification_channels.unconfigured")}
                                                }
                                            }
                                        }
                                        if channel.configured {
                                            Badge { variant: BadgeVariant::Success, "Configured" }
                                        } else {
                                            Badge { variant: BadgeVariant::Secondary, "Off" }
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
