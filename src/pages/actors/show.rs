use dioxus::prelude::*;

use crate::api::actors;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::icons::Icon;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::page_header::{Breadcrumbs, PageHeader, BreadcrumbItem};
use crate::router::Route;
use crate::types::UpdateActorRequest;
use crate::utils::i18n::t;

#[component]
pub fn ActorShow(actor_id: String) -> Element {
    let mut suspending = use_signal(|| false);
    let actor_id_clone = actor_id.clone();
    let mut data = use_resource(move || {
        let id = actor_id_clone.clone();
        async move { actors::get_actor(&id).await }
    });

    let _nav = use_navigator();

    let actor_id_for_suspend = actor_id.clone();
    let actor_id_for_unsuspend = actor_id.clone();
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

                        div { class: "grid gap-6 md:grid-cols-2",
                            Card {
                                CardHeader { CardTitle { {t("actors.overview")} } }
                                CardContent {
                                    div { class: "space-y-3",
                                    {field_row(t("actors.id"), actor.id.clone())}
                                    {field_row(t("actors.did"), actor.did.clone())}
                                    {field_row(t("actors.handle"), actor.handle.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("actors.display_name"), actor.display_name.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("actors.status"), if actor.is_suspended { t("actors.suspended") } else if actor.is_deactivated { t("actors.deactivated") } else { t("actors.active") })}
                                    {field_row(t("actors.is_admin"), if actor.is_admin { t("common.yes") } else { t("common.no") })}
                                    {field_row(t("actors.created_at"), actor.created_at.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("actors.last_active"), actor.last_active_at.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("actors.device_count"), actor.device_count.to_string())}
                                    {field_row(t("actors.space_count"), actor.space_count.to_string())}
                                    }
                                }
                            }

                            Card {
                                CardHeader { CardTitle { {t("actors.actions")} } }
                                CardContent {
                                    div { class: "space-y-2",
                                        if !actor.is_suspended && !actor.is_deactivated {
                                            Button {
                                                class: "w-full".to_string(),
                                                disabled: suspending(),
                                                onclick: move |_| {
                                                    let aid = actor_id_for_suspend.clone();
                                                    suspending.set(true);
                                                    spawn(async move {
                                                        let req = UpdateActorRequest { is_suspended: Some(true), ..Default::default() };
                                                        let _ = actors::update_actor(&aid, &req).await;
                                                        suspending.set(false);
                                                        data.restart();
                                                    });
                                                },
                                                {t("actors.suspend")}
                                            }
                                        }
                                        if actor.is_suspended {
                                            Button {
                                                class: "w-full".to_string(),
                                                disabled: suspending(),
                                                onclick: move |_| {
                                                    let aid = actor_id_for_unsuspend.clone();
                                                    suspending.set(true);
                                                    spawn(async move {
                                                        let req = UpdateActorRequest { is_suspended: Some(false), ..Default::default() };
                                                        let _ = actors::update_actor(&aid, &req).await;
                                                        suspending.set(false);
                                                        data.restart();
                                                    });
                                                },
                                                {t("actors.unsuspend")}
                                            }
                                        }
                                        if !actor.is_deactivated {
                                            Button {
                                                variant: ButtonVariant::Destructive,
                                                class: "w-full".to_string(),
                                                onclick: move |_| {
                                                    let aid = actor_id_for_deactivate.clone();
                                                    spawn(async move {
                                                        let _ = actors::deactivate_actor(&aid).await;
                                                        data.restart();
                                                    });
                                                },
                                                {t("actors.deactivate")}
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
