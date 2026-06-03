//! Soland Spaces admin list
//!
//! Cursor-paginated list of Spaces visible to the current admin scope,
//! backed by `GET /admin/spaces`. Each row shows name, member
//! count, health (Active / Frozen / Destroyed) and an "Open detail"
//! button that links to the existing per-space detail page.
//!
//! Distinct from `pages/spaces/list.rs` (the user-facing Space list) —
//! this page integrates the soland admin describe surface and exposes
//! the health badge that drives the destroy/freeze workflows.
//! 404-tolerant on the client side.

use dioxus::prelude::*;

use crate::api::spaces_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::SearchInput;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::router::Route;
use crate::types::spaces_admin::SpaceHealth;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn SpaceAdminList() -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut search = use_signal(String::new);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let search_snapshot = search.read().clone();

    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        let s = search_snapshot.clone();
        async move { spaces_admin::list_admin_spaces(cursor.as_deref(), PAGE_SIZE, &s).await }
    });

    let mut reset_to_first_page = move || {
        cursor_stack.set(vec![None]);
    };

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("spaces_admin.title"),
                description: t("spaces_admin.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            div { class: "max-w-md",
                SearchInput {
                    value: search.read().clone(),
                    placeholder: t("spaces_admin.search_placeholder"),
                    oninput: move |evt: FormEvent| {
                        reset_to_first_page();
                        search.set(evt.value());
                    },
                }
            }

            match &*data.read() {
                Some(Ok(page)) => {
                    let next_cursor = page.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    let total_label = match page.total {
                        Some(n) => format!("{n}"),
                        None => "?".to_string(),
                    };
                    let row_count = page.data.len();
                    rsx! {
                        if page.data.is_empty() {
                            EmptyState {
                                icon: "message-square".to_string(),
                                title: t("spaces_admin.empty_title"),
                                description: t("spaces_admin.empty_subtitle"),
                            }
                        } else {
                            p { class: "text-xs text-muted-foreground",
                                {format!("Showing {row_count} (server total: {total_label})")}
                            }
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("spaces_admin.id")} }
                                            TableHead { {t("spaces_admin.name")} }
                                            TableHead { {t("spaces_admin.members")} }
                                            TableHead { {t("spaces_admin.health")} }
                                            TableHead { {t("spaces_admin.created_at")} }
                                            TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                        }
                                    }
                                    TableBody {
                                        for row in page.data.iter() {
                                            {
                                                let space_id = row.id.clone();
                                                let name = row.name.clone().unwrap_or_else(|| "-".to_string());
                                                let members = row.member_count;
                                                let typed = row.health_typed();
                                                let label = typed.label().to_string();
                                                let variant = health_badge_variant(&typed);
                                                let created = row
                                                    .created_at
                                                    .clone()
                                                    .unwrap_or_else(|| "-".to_string());
                                                let detail_route = Route::SpaceShow {
                                                    space_id: space_id.clone(),
                                                };
                                                let hierarchy_route = Route::SpaceHierarchy {
                                                    space_id: space_id.clone(),
                                                };
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{space_id}" }
                                                        TableCell { "{name}" }
                                                        TableCell { "{members}" }
                                                        TableCell {
                                                            Badge { variant, "{label}" }
                                                        }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            div { class: "flex justify-end gap-2",
                                                                Link {
                                                                    to: hierarchy_route,
                                                                    class: "inline-flex h-8 items-center rounded-md border px-2 text-xs font-medium transition-colors hover:bg-accent".to_string(),
                                                                    {t("spaces_admin.open_hierarchy")}
                                                                }
                                                                Link {
                                                                    to: detail_route,
                                                                    class: "inline-flex h-8 items-center rounded-md border px-2 text-xs font-medium transition-colors hover:bg-accent".to_string(),
                                                                    {t("spaces_admin.open_detail")}
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

                            div { class: "flex items-center justify-between px-2 py-4",
                                div { class: "text-sm text-muted-foreground",
                                    {format!("Page {}", stack_depth)}
                                }
                                div { class: "flex items-center space-x-2",
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        size: ButtonSize::Sm,
                                        disabled: stack_depth <= 1,
                                        onclick: move |_| {
                                            let mut new_stack = cursor_stack.read().clone();
                                            if new_stack.len() > 1 {
                                                new_stack.pop();
                                                cursor_stack.set(new_stack);
                                            }
                                        },
                                        {t("common.previous")}
                                    }
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        size: ButtonSize::Sm,
                                        disabled: next_cursor.is_none(),
                                        onclick: move |_| {
                                            if let Some(c) = next_cursor.clone() {
                                                let mut new_stack = cursor_stack.read().clone();
                                                new_stack.push(Some(c));
                                                cursor_stack.set(new_stack);
                                            }
                                        },
                                        {t("common.next")}
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

/// Pick a Badge variant for a Space health value. `Active` is success
/// (green; happy path), `Frozen` is the neutral secondary tone
/// (operational hold), `Destroyed` is destructive (red; tombstoned).
/// Pure helper so the mapping is unit-testable.
pub(crate) fn health_badge_variant(health: &SpaceHealth) -> BadgeVariant {
    match health {
        SpaceHealth::Active => BadgeVariant::Success,
        SpaceHealth::Frozen => BadgeVariant::Secondary,
        SpaceHealth::Destroyed => BadgeVariant::Destructive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_variant_buckets_match_severity() {
        assert!(matches!(
            health_badge_variant(&SpaceHealth::Active),
            BadgeVariant::Success
        ));
        assert!(matches!(
            health_badge_variant(&SpaceHealth::Frozen),
            BadgeVariant::Secondary
        ));
        assert!(matches!(
            health_badge_variant(&SpaceHealth::Destroyed),
            BadgeVariant::Destructive
        ));
    }
}
