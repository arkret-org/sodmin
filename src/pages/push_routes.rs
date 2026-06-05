//! T6.2 §4 — `ck.device.push_route` admin inspector.
//!
//! Surfaces all push routes for a given principal. The 4-tuple
//! `(principal_id, device_id, transport, route_id)` is shown for each
//! row; `push_target_id` is sensitive and stays collapsed behind a
//! per-row reveal toggle.

use std::collections::HashSet;

use dioxus::prelude::*;

use crate::api::push_routes;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::SearchInput;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::utils::i18n::t;
use crate::utils::net::error::should_reset_cursor_pagination;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn PushRoutes() -> Element {
    let mut principal_id = use_signal(String::new);
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut revealed = use_signal(HashSet::<String>::new);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let cursor_for_fetch = cursor_snapshot.clone();
    let principal_val = principal_id.read().clone();

    let mut data = use_resource(move || {
        let cursor = cursor_for_fetch.clone();
        let p = principal_val.clone();
        async move { push_routes::list_push_routes(cursor.as_deref(), PAGE_SIZE, &p).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("push_routes.title"),
                description: t("push_routes.subtitle"),
            }

            SearchInput {
                placeholder: t("push_routes.search"),
                value: principal_id.read().clone(),
                oninput: move |evt: FormEvent| {
                    principal_id.set(evt.value());
                    cursor_stack.set(vec![None::<String>]);
                },
            }

            match &*data.read() {
                Some(Ok(resp)) => {
                    let next_cursor = resp.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    rsx! {
                        div { class: "rounded-md border",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead { {t("push_routes.principal")} }
                                        TableHead { {t("push_routes.device")} }
                                        TableHead { {t("push_routes.transport")} }
                                        TableHead { {t("push_routes.cell_subject")} }
                                        TableHead { {t("push_routes.status")} }
                                        TableHead { {t("push_routes.last_rotation")} }
                                        TableHead { class: "text-right".to_string(), {t("push_routes.target")} }
                                    }
                                }
                                TableBody {
                                    if resp.data.is_empty() {
                                        TableRow {
                                            TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                {t("push_routes.empty")}
                                            }
                                        }
                                    } else {
                                        for route in resp.data.iter() {
                                            {
                                                let row_key = format!("{}::{}", route.principal_id, route.cell_subject);
                                                let revealed_now = revealed.read().contains(&row_key);
                                                let principal = route.principal_id.clone();
                                                let device = route.device_id.clone();
                                                let transport = route.transport.clone().unwrap_or_else(|| "-".to_string());
                                                let status = route.status.clone().unwrap_or_else(|| "-".to_string());
                                                let is_active = status.eq_ignore_ascii_case("active");
                                                let last_rotation = route.last_rotation_at.clone().unwrap_or_else(|| "-".to_string());
                                                let tuple_display = if route.cell_subject_tuple.is_empty() {
                                                    route.cell_subject.clone()
                                                } else {
                                                    route.cell_subject_tuple.join(" / ")
                                                };
                                                let push_target = route.push_target_id.clone().unwrap_or_default();
                                                let key_for_toggle = row_key.clone();

                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs max-w-[160px] truncate".to_string(), "{principal}" }
                                                        TableCell { class: "font-mono text-xs max-w-[160px] truncate".to_string(), "{device}" }
                                                        TableCell { class: "text-xs".to_string(), "{transport}" }
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{tuple_display}" }
                                                        TableCell {
                                                            if is_active {
                                                                Badge { variant: BadgeVariant::Success, "{status}" }
                                                            } else {
                                                                Badge { variant: BadgeVariant::Destructive, "{status}" }
                                                            }
                                                        }
                                                        TableCell { class: "text-xs text-muted-foreground".to_string(), "{last_rotation}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            if revealed_now {
                                                                div { class: "flex items-center justify-end gap-2",
                                                                    code { class: "text-xs font-mono break-all max-w-[200px] truncate", "{push_target}" }
                                                                    Button {
                                                                        variant: ButtonVariant::Ghost,
                                                                        size: ButtonSize::Sm,
                                                                        onclick: {
                                                                            let key = key_for_toggle.clone();
                                                                            move |_| {
                                                                                let mut set = revealed.read().clone();
                                                                                set.remove(&key);
                                                                                revealed.set(set);
                                                                            }
                                                                        },
                                                                        {t("push_routes.hide")}
                                                                    }
                                                                }
                                                            } else {
                                                                Button {
                                                                    variant: ButtonVariant::Outline,
                                                                    size: ButtonSize::Sm,
                                                                    onclick: {
                                                                        let key = key_for_toggle.clone();
                                                                        move |_| {
                                                                            let mut set = revealed.read().clone();
                                                                            set.insert(key.clone());
                                                                            revealed.set(set);
                                                                        }
                                                                    },
                                                                    {t("push_routes.reveal")}
                                                                }
                                                            }
                                                        }
                                                    }
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
                Some(Err(e)) => {
                    let reset_cursor = should_reset_cursor_pagination(e, cursor_snapshot.as_deref());
                    rsx! {
                        ErrorBanner {
                            message: e.message.clone(),
                            on_retry: move |_| {
                                if reset_cursor {
                                    cursor_stack.set(vec![None::<String>]);
                                }
                                data.restart();
                            },
                        }
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
