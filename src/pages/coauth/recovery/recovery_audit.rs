//! Recovery audit-log viewer (Round 25, C4).
//!
//! Cursor-paginated read-only feed of admin actions taken on recovery
//! tickets. The page is keyed by an optional `ticket_id` query
//! parameter so operators can pivot from the detail page (C2) into a
//! filtered audit feed for that ticket.

use dioxus::prelude::*;

use crate::api::recovery_admin;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 50;

#[component]
pub fn RecoveryAuditLogPage(ticket_id: String) -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let id_for_resource = ticket_id.clone();

    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        let id = id_for_resource.clone();
        async move { recovery_admin::list_audit(cursor.as_deref(), PAGE_SIZE, &id).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("recovery_audit.title"),
                description: t("recovery_audit.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            if !ticket_id.is_empty() {
                p { class: "text-xs text-muted-foreground",
                    {format!("{}: {}", t("recovery_audit.filter_ticket_id"), ticket_id)}
                }
            }

            match &*data.read() {
                Some(Ok(page)) => {
                    let next_cursor = page.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    rsx! {
                        if page.data.is_empty() {
                            EmptyState {
                                icon: "scroll-text".to_string(),
                                title: t("recovery_audit.empty_title"),
                                description: t("recovery_audit.empty_subtitle"),
                            }
                        } else {
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("recovery_audit.at")} }
                                            TableHead { {t("recovery_audit.action")} }
                                            TableHead { {t("recovery_audit.ticket_id")} }
                                            TableHead { {t("recovery_audit.actor_did")} }
                                            TableHead { {t("recovery_audit.note")} }
                                        }
                                    }
                                    TableBody {
                                        for entry in page.data.iter() {
                                            {
                                                let at = entry.at.clone().unwrap_or_else(|| "-".to_string());
                                                let action = entry.action.clone();
                                                let tid = entry.ticket_id.clone().unwrap_or_default();
                                                let actor = entry.actor_did.clone().unwrap_or_default();
                                                let note = entry.note.clone().unwrap_or_default();
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs".to_string(), "{at}" }
                                                        TableCell { class: "font-medium".to_string(), "{action}" }
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{tid}" }
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{actor}" }
                                                        TableCell { class: "max-w-[300px] truncate".to_string(), "{note}" }
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

/// Pure helper — picks a human-readable label for an action wire
/// string. Kept here so it stays unit-testable.
pub(crate) fn audit_action_label(wire: &str) -> &'static str {
    match wire {
        "approve" => "Approved",
        "reject" => "Rejected",
        "advance" => "Advanced",
        "cancel" => "Cancelled",
        "executor_started" => "Executor started",
        "executor_completed" => "Executor completed",
        "executor_failed" => "Executor failed",
        _ => "Other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_label_buckets() {
        assert_eq!(audit_action_label("approve"), "Approved");
        assert_eq!(audit_action_label("reject"), "Rejected");
        assert_eq!(audit_action_label("advance"), "Advanced");
        assert_eq!(audit_action_label("cancel"), "Cancelled");
        assert_eq!(audit_action_label("executor_failed"), "Executor failed");
        assert_eq!(audit_action_label("garbage"), "Other");
    }
}
