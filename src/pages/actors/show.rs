use dioxus::prelude::*;

use crate::api::actors;
use crate::components::dangerous_action_dialog::{DangerousActionDialog, confirmation_suffix};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::types::AdminActorExt;
use crate::utils::i18n::t;
use crate::utils::net::audit::{AdminAuditOutcome, emit_admin_audit_server};

#[component]
pub fn ActorShow(actor_id: String) -> Element {
    let mut show_deactivate_dialog = use_signal(|| false);
    let actor_id_clone = actor_id.clone();
    let mut data = use_resource(move || {
        let id = actor_id_clone.clone();
        async move { actors::get_actor(&id).await }
    });

    let actor_id_for_deactivate = actor_id.clone();

    rsx! {
        div { class: "space-y-6",
            match &*data.read() {
                Some(Ok(actor)) => {
                    let breadcrumbs = vec![
                        BreadcrumbItem { label: t("nav.actors"), route: Some(Route::ActorList {}) },
                        BreadcrumbItem { label: actor.display_name.as_deref().unwrap_or(&actor.id).to_string(), route: None },
                    ];
                    rsx! {
                        Breadcrumbs {
                            items: breadcrumbs,
                        }

                        PageHeader {
                            title: actor.display_name.as_deref().unwrap_or(&actor.id).to_string(),
                        }

                        // Round 4 — when the local 7-domain fanout
                        // succeeded but federation peers have not all
                        // confirmed, surface the in-progress state
                        // explicitly. Silently rendering "Deactivated"
                        // here would mislead operators into believing
                        // the principal is fully gone everywhere.
                        if actor.is_deactivated() && actor.deactivation_federation_incomplete == Some(true) {
                            div {
                                class: "rounded-md border-2 border-amber-600 bg-amber-600/10 px-3 py-2 text-sm space-y-1",
                                role: "alert",
                                p { class: "font-semibold text-amber-700 dark:text-amber-200",
                                    span { class: "mr-2", "\u{26A0}" }
                                    {t("actors.deactivation_federation_incomplete")}
                                }
                                p { class: "text-xs text-amber-700/90 dark:text-amber-200/90",
                                    {t("actors.deactivation_federation_incomplete_detail")}
                                }
                            }
                        }

                        div { class: "grid gap-6 md:grid-cols-2",
                            Card {
                                CardHeader { CardTitle { {t("actors.overview")} } }
                                CardContent {
                                    div { class: "space-y-3",
                                    {field_row(t("actors.id"), actor.id.clone())}
                                    {field_row(t("actors.did"), actor.did.to_string())}
                                    {field_row(t("actors.handle"), actor.handle.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("actors.display_name"), actor.display_name.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("actors.status"), actor.status_display().to_string())}
                                    {field_row(t("actors.is_admin"), match actor.is_admin {
                                        Some(true) => t("common.yes"),
                                        Some(false) => t("common.no"),
                                        None => "-".to_string(),
                                    })}
                                    {field_row(t("actors.created_at"), actor.created_at.map(|ts| ts.to_rfc3339()).unwrap_or_else(|| "-".to_string()))}
                                    {field_row(t("actors.last_active"), actor.last_active_at.map(|ts| ts.to_rfc3339()).unwrap_or_else(|| "-".to_string()))}
                                    {field_row(t("actors.device_count"), actor.device_count.map(|count| count.to_string()).unwrap_or_else(|| "-".to_string()))}
                                    {field_row(t("actors.realm_count"), actor.realm_count.map(|count| count.to_string()).unwrap_or_else(|| "-".to_string()))}
                                    }
                                }
                            }

                            Card {
                                CardHeader { CardTitle { {t("actors.actions")} } }
                                CardContent {
                                    div { class: "space-y-2",
                                        if !actor.is_deactivated() {
                                            Button {
                                                variant: ButtonVariant::Destructive,
                                                class: "w-full".to_string(),
                                                onclick: move |_| show_deactivate_dialog.set(true),
                                                {t("actors.deactivate")}
                                            }
                                        } else {
                                            p { class: "text-sm text-muted-foreground",
                                                {t("actors.already_deactivated")}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { ErrorBanner { message: e.message.clone(), on_retry: move |_| data.restart() } },
                None => rsx! { PageSkeleton {} },
            }

            // Deactivation is an irreversible seven-domain fanout — gate it
            // behind the same typed-phrase confirmation the rest of the
            // destructive surface uses (type the last 4 chars of the
            // actor id), then surface success/failure + audit breadcrumb.
            {
                let aid = actor_id_for_deactivate.clone();
                let phrase = confirmation_suffix(&aid, 4);
                rsx! {
                    DangerousActionDialog {
                        open: show_deactivate_dialog(),
                        title: t("actors.deactivate"),
                        description: format!("{} ({})", t("actors.deactivate"), aid),
                        confirmation_phrase: phrase,
                        confirm_text: t("actors.deactivate"),
                        cancel_text: t("common.cancel"),
                        on_cancel: move |_| show_deactivate_dialog.set(false),
                        on_confirm: move |_| {
                            let aid = aid.clone();
                            show_deactivate_dialog.set(false);
                            spawn(async move {
                                // Account-lifecycle endpoints address accounts by
                                // DID — the route's actor id IS the DID.
                                match actors::deactivate_account(&aid).await {
                                    Ok(_) => {
                                        show_toast(&t("actors.deactivated"), ToastVariant::Success);
                                        emit_admin_audit_server("account", &aid, "deactivate", AdminAuditOutcome::Accepted, None);
                                    }
                                    Err(e) => {
                                        show_toast(&e.message, ToastVariant::Error);
                                        emit_admin_audit_server("account", &aid, "deactivate", AdminAuditOutcome::Rejected, Some(&e.message));
                                    }
                                }
                                data.restart();
                            });
                        },
                    }
                }
            }
        }
    }
}

fn field_row(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex justify-between py-1.5 border-b border-border/50 last:border-0",
            span { class: "text-sm text-muted-foreground", "{label}" }
            span { class: "text-sm font-medium", "{value}" }
        }
    }
}
