use dioxus::prelude::*;

use crate::api::actors;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::SearchInput;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::router::Route;
use crate::types::{AccountStatus, AdminActorExt};
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 20;

pub(crate) fn actor_status_badge_variant(status: Option<AccountStatus>) -> BadgeVariant {
    match status {
        Some(AccountStatus::Active) => BadgeVariant::Success,
        Some(AccountStatus::SoftLoggedOut | AccountStatus::Locked) => BadgeVariant::Secondary,
        Some(
            AccountStatus::Suspended | AccountStatus::Deactivated | AccountStatus::ErasurePending,
        ) => BadgeVariant::Destructive,
        None => BadgeVariant::Secondary,
    }
}

#[component]
pub fn ActorList() -> Element {
    let mut search = use_signal(String::new);
    let mut cursor_stack = use_signal(|| vec![None::<String>]);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let search_val = search.read().clone();
    let mut data = use_resource(move || {
        let s = search_val.clone();
        let cursor = cursor_snapshot.clone();
        async move { actors::list_actors(cursor.as_deref(), PAGE_SIZE, &s).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("nav.actors"),
                description: t("actors.description"),
            }

            div { class: "flex items-center gap-4",
                SearchInput {
                    value: search(),
                    placeholder: t("actors.search_placeholder"),
                    oninput: move |evt: FormEvent| {
                        search.set(evt.value());
                        cursor_stack.set(vec![None::<String>]);
                    },
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => {
                    let next_cursor = resp.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    rsx! {
                        div { class: "rounded-lg border glass-panel overflow-hidden",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead { {t("actors.id")} }
                                        TableHead { {t("actors.handle")} }
                                        TableHead { {t("actors.display_name")} }
                                        TableHead { {t("actors.status")} }
                                        TableHead { {t("actors.is_admin")} }
                                        TableHead { {t("actors.created_at")} }
                                    }
                                }
                                TableBody {
                                    if resp.actors.is_empty() {
                                        TableRow {
                                            TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                {t("common.no_results")}
                                            }
                                        }
                                    }
                                    for actor in resp.actors.iter() {
                                        {
                                            let actor_id = actor.id.clone();
                                            let status_label = actor.status_display();
                                            let status_variant = actor_status_badge_variant(actor.status);
                                            let is_admin = match actor.is_admin {
                                                Some(true) => t("common.yes"),
                                                Some(false) => t("common.no"),
                                                None => "-".to_string(),
                                            };
                                            let created_at = actor
                                                .created_at
                                                .map(|ts| ts.to_rfc3339())
                                                .unwrap_or_else(|| "-".to_string());
                                            rsx! {
                                                TableRow {
                                                    key: "{actor.id}",
                                                    TableCell { class: "font-mono text-xs",
                                                        Link {
                                                            to: Route::ActorShow { actor_id: actor_id.clone() },
                                                            class: "hover:underline",
                                                            "{actor.id}"
                                                        }
                                                    }
                                                    TableCell { {actor.handle.as_deref().unwrap_or("-")} }
                                                    TableCell { {actor.display_name.as_deref().unwrap_or("-")} }
                                                    TableCell { Badge { variant: status_variant, "{status_label}" } }
                                                    TableCell { {is_admin} }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{created_at}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        CursorPagination {
                            depth: stack_depth,
                            has_next: next_cursor.is_some(),
                            on_prev: move |_| {
                                let mut new_stack = cursor_stack.read().clone();
                                if new_stack.len() > 1 {
                                    new_stack.pop();
                                    cursor_stack.set(new_stack);
                                }
                            },
                            on_next: move |_| {
                                if let Some(c) = next_cursor.clone() {
                                    let mut new_stack = cursor_stack.read().clone();
                                    new_stack.push(Some(c));
                                    cursor_stack.set(new_stack);
                                }
                            },
                        }
                    }
                },
                Some(Err(e)) => rsx! { ErrorBanner { message: e.message.clone(), on_retry: move |_| data.restart() } },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
