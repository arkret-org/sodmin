//! Read-only soland recovery describe surface (Round 25, C5).
//!
//! Mirrors the per-bridge describe object — supported recovery modes,
//! verification event kinds, and the path table the bridge advertises.
//! No mutations; this is the canonical view for operators auditing
//! what the soland bridge claims to support.

use dioxus::prelude::*;

use crate::api::recovery_admin;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::utils::i18n::t;

#[component]
pub fn RecoveryDescribePage() -> Element {
    let mut data = use_resource(move || async move { recovery_admin::describe().await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("recovery_describe.title"),
                description: t("recovery_describe.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            match &*data.read() {
                Some(Ok(d)) => {
                    let modes = d.recovery_modes.join(", ");
                    let verification = d.verification_event_kinds.join(", ");
                    rsx! {
                        div { class: "rounded-md border p-4 space-y-2",
                            div {
                                strong { {t("recovery_describe.contract")} ":" }
                                span { class: "ml-2 font-mono", "{d.contract}" }
                            }
                            div {
                                strong { {t("recovery_describe.version")} ":" }
                                span { class: "ml-2 font-mono", "{d.version}" }
                            }
                            div {
                                strong { {t("recovery_describe.recovery_modes")} ":" }
                                span { class: "ml-2", "{modes}" }
                            }
                            div {
                                strong { {t("recovery_describe.verification_kinds")} ":" }
                                span { class: "ml-2", "{verification}" }
                            }
                        }
                        h2 { class: "text-base font-semibold", {t("recovery_describe.paths_title")} }
                        if d.paths.is_empty() {
                            p { class: "text-sm text-muted-foreground", {t("recovery_describe.paths_empty")} }
                        } else {
                            ul { class: "rounded-md border divide-y",
                                for p in d.paths.iter() {
                                    li { class: "flex items-center justify-between p-3 text-sm",
                                        span { class: "font-medium", "{p.label}" }
                                        span { class: "font-mono text-xs text-muted-foreground", "{p.path}" }
                                    }
                                }
                            }
                        }
                    }
                }
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
