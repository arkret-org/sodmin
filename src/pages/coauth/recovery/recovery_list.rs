//! Recovery tickets list (Round 25, C1).
//!
//! Cursor-paginated list of recovery tickets with a status-filter
//! bucket bar. Defaults to `active` (everything that isn't a terminal
//! state). Each row links to the per-ticket detail page (C2).

use dioxus::prelude::*;

use crate::api::recovery_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::router::Route;
use crate::utils::i18n::t;
use coauth_admin_types::recovery_admin::RecoveryTicketStatus;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn RecoveryTicketListPage() -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut status_filter = use_signal(|| "active".to_string());

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let status_snapshot = status_filter.read().clone();

    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        let status = status_snapshot.clone();
        async move { recovery_admin::list_tickets(cursor.as_deref(), PAGE_SIZE, &status).await }
    });

    let mut reset_to_first_page = move || {
        cursor_stack.set(vec![None]);
    };

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("recovery_list.title"),
                description: t("recovery_list.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            div { class: "flex gap-2",
                {
                    let buckets: [(&str, &str); 5] = [
                        ("active", "recovery_list.filter_active"),
                        ("pending", "recovery_list.filter_pending"),
                        ("approved", "recovery_list.filter_approved"),
                        ("complete", "recovery_list.filter_complete"),
                        ("all", "recovery_list.filter_all"),
                    ];
                    rsx! {
                        for (bucket, key) in buckets.iter() {
                            {
                                let bucket_owned = bucket.to_string();
                                let active = *status_filter.read() == bucket_owned;
                                let label = t(key);
                                rsx! {
                                    Button {
                                        variant: if active { ButtonVariant::Default } else { ButtonVariant::Outline },
                                        size: ButtonSize::Sm,
                                        onclick: move |_| {
                                            reset_to_first_page();
                                            status_filter.set(bucket_owned.clone());
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            match &*data.read() {
                Some(Ok(page)) => {
                    let next_cursor = page.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    let row_count = page.data.len();
                    rsx! {
                        if page.data.is_empty() {
                            EmptyState {
                                icon: "shield".to_string(),
                                title: t("recovery_list.empty_title"),
                                description: t("recovery_list.empty_subtitle"),
                            }
                        } else {
                            p { class: "text-xs text-muted-foreground",
                                {format!("Showing {row_count}")}
                            }
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("recovery_list.ticket_id")} }
                                            TableHead { {t("recovery_list.account_did")} }
                                            TableHead { {t("recovery_list.status")} }
                                            TableHead { {t("recovery_list.mode")} }
                                            TableHead { {t("recovery_list.created_at")} }
                                            TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                        }
                                    }
                                    TableBody {
                                        for r in page.data.iter() {
                                            {
                                                let ticket_id = r.ticket_id.clone();
                                                let account = r.account_did.clone();
                                                let typed = r.status_typed();
                                                let label = typed.label().to_string();
                                                let variant = ticket_status_variant(&typed);
                                                let mode = r.mode.clone().unwrap_or_else(|| "-".to_string());
                                                let created = r.created_at.clone().unwrap_or_else(|| "-".to_string());
                                                let detail_route = Route::RecoveryTicketDetail {
                                                    ticket_id: ticket_id.clone(),
                                                };
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{ticket_id}" }
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{account}" }
                                                        TableCell { Badge { variant, "{label}" } }
                                                        TableCell { "{mode}" }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            Link {
                                                                to: detail_route,
                                                                class: "inline-flex h-8 items-center rounded-md border px-2 text-xs font-medium transition-colors hover:bg-accent".to_string(),
                                                                {t("common.open")}
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

/// Pick a Badge variant for a recovery ticket status. Pure helper so
/// the mapping is unit-testable without a browser.
pub(crate) fn ticket_status_variant(s: &RecoveryTicketStatus) -> BadgeVariant {
    match s {
        RecoveryTicketStatus::Pending => BadgeVariant::Default,
        RecoveryTicketStatus::Approved => BadgeVariant::Secondary,
        RecoveryTicketStatus::ExecutorRunning => BadgeVariant::Default,
        RecoveryTicketStatus::Complete => BadgeVariant::Success,
        RecoveryTicketStatus::Cancelled => BadgeVariant::Secondary,
        RecoveryTicketStatus::Rejected => BadgeVariant::Destructive,
        RecoveryTicketStatus::Failed => BadgeVariant::Destructive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_buckets_by_severity() {
        assert!(matches!(
            ticket_status_variant(&RecoveryTicketStatus::Complete),
            BadgeVariant::Success
        ));
        assert!(matches!(
            ticket_status_variant(&RecoveryTicketStatus::Rejected),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            ticket_status_variant(&RecoveryTicketStatus::Failed),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            ticket_status_variant(&RecoveryTicketStatus::Approved),
            BadgeVariant::Secondary
        ));
        assert!(matches!(
            ticket_status_variant(&RecoveryTicketStatus::Cancelled),
            BadgeVariant::Secondary
        ));
    }

    #[test]
    fn pending_and_running_are_neutral_default_tone() {
        assert!(matches!(
            ticket_status_variant(&RecoveryTicketStatus::Pending),
            BadgeVariant::Default
        ));
        assert!(matches!(
            ticket_status_variant(&RecoveryTicketStatus::ExecutorRunning),
            BadgeVariant::Default
        ));
    }
}
