//! Agents admin (Round 26, F2 — full mutation flow).
//!
//! Cursor-paginated list of soland-side agent registrations with
//! per-row Approve / Suspend / Revoke buttons. ConfirmDialog destructive
//! on Suspend / Revoke. 404-tolerant. Each click emits a structured
//! client-side audit line via `utils::audit::emit_admin_audit`.

use dioxus::prelude::*;

use crate::api::applets_agents_directory_admin as admin_api;
use crate::components::ui::badge::Badge;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::pages::applets::admin::{RowAction, approval_variant};
use coauth_admin_types::applets_admin::ApprovalActionRequest;
use crate::utils::audit::{AdminAuditOutcome, emit_admin_audit};
use crate::utils::error::format_optional_endpoint_error;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[derive(Debug, Clone)]
struct PendingDecision {
    id: String,
    action: RowAction,
}

#[component]
pub fn AgentAdminPage() -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut pending = use_signal::<Option<PendingDecision>>(|| None);
    let mut in_flight = use_signal::<Option<String>>(|| None);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);

    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        async move { admin_api::list_agents_admin(cursor.as_deref(), PAGE_SIZE).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("agents_admin.title"),
                description: t("agents_admin.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            match &*data.read() {
                Some(Ok(page)) => {
                    let next_cursor = page.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    rsx! {
                        if page.data.is_empty() {
                            EmptyState {
                                icon: "bot".to_string(),
                                title: t("agents_admin.empty_title"),
                                description: t("agents_admin.empty_subtitle"),
                            }
                        } else {
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("agents_admin.id")} }
                                            TableHead { {t("agents_admin.name")} }
                                            TableHead { {t("agents_admin.owner_did")} }
                                            TableHead { {t("agents_admin.status")} }
                                            TableHead { {t("agents_admin.registered_at")} }
                                            TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                        }
                                    }
                                    TableBody {
                                        for r in page.data.iter() {
                                            {
                                                let id = r.id.clone();
                                                let name = r.name.clone().unwrap_or_else(|| "-".to_string());
                                                let owner = r.owner_did.clone().unwrap_or_else(|| "-".to_string());
                                                let typed = r.status_typed();
                                                let label = typed.label().to_string();
                                                let variant = approval_variant(&typed);
                                                let registered = r.registered_at.clone().unwrap_or_else(|| "-".to_string());
                                                let approvable = typed.is_approvable();
                                                let suspendable = typed.is_suspendable();
                                                let revocable = typed.is_revocable();
                                                let id_a = id.clone();
                                                let id_s = id.clone();
                                                let id_r = id.clone();
                                                let row_busy = in_flight
                                                    .read()
                                                    .as_deref()
                                                    .map(|x| x == id.as_str())
                                                    .unwrap_or(false);
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{id}" }
                                                        TableCell { "{name}" }
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{owner}" }
                                                        TableCell { Badge { variant, "{label}" } }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{registered}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            div { class: "flex justify-end gap-2",
                                                                Button {
                                                                    variant: ButtonVariant::Default,
                                                                    size: ButtonSize::Sm,
                                                                    disabled: !approvable || row_busy,
                                                                    onclick: move |_| {
                                                                        pending.set(Some(PendingDecision {
                                                                            id: id_a.clone(),
                                                                            action: RowAction::Approve,
                                                                        }));
                                                                    },
                                                                    {t("common.approve")}
                                                                }
                                                                Button {
                                                                    variant: ButtonVariant::Outline,
                                                                    size: ButtonSize::Sm,
                                                                    disabled: !suspendable || row_busy,
                                                                    onclick: move |_| {
                                                                        pending.set(Some(PendingDecision {
                                                                            id: id_s.clone(),
                                                                            action: RowAction::Suspend,
                                                                        }));
                                                                    },
                                                                    {t("common.suspend")}
                                                                }
                                                                Button {
                                                                    variant: ButtonVariant::Destructive,
                                                                    size: ButtonSize::Sm,
                                                                    disabled: !revocable || row_busy,
                                                                    onclick: move |_| {
                                                                        pending.set(Some(PendingDecision {
                                                                            id: id_r.clone(),
                                                                            action: RowAction::Revoke,
                                                                        }));
                                                                    },
                                                                    {t("common.revoke")}
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

            {
                let p = pending.read().clone();
                let action = p.as_ref().map(|x| x.action).unwrap_or(RowAction::Approve);
                let (title_key, body_key, confirm_label) = agent_dialog_copy(action);
                rsx! {
                    ConfirmDialog {
                        open: p.is_some(),
                        title: t(title_key),
                        description: t(body_key),
                        confirm_text: t(confirm_label),
                        cancel_text: t("common.cancel"),
                        destructive: action.is_destructive(),
                        on_cancel: move |_| pending.set(None),
                        on_confirm: move |_| {
                            if let Some(p) = pending.read().clone() {
                                in_flight.set(Some(p.id.clone()));
                                spawn(async move {
                                    let body = ApprovalActionRequest::default();
                                    let res = match p.action {
                                        RowAction::Approve => admin_api::approve_agent(&p.id, &body).await,
                                        RowAction::Suspend => admin_api::suspend_agent(&p.id, &body).await,
                                        RowAction::Revoke => admin_api::revoke_agent(&p.id, &body).await,
                                    };
                                    match res {
                                        Ok(_) => {
                                            emit_admin_audit(
                                                "agent",
                                                &p.id,
                                                p.action.wire(),
                                                AdminAuditOutcome::Accepted,
                                                None,
                                            );
                                            show_toast(
                                                "Agent decision recorded.",
                                                ToastVariant::Success,
                                            );
                                        }
                                        Err(e) => {
                                            emit_admin_audit(
                                                "agent",
                                                &p.id,
                                                p.action.wire(),
                                                AdminAuditOutcome::from_http_status(e.status),
                                                None,
                                            );
                                            let msg = format_optional_endpoint_error(
                                                agent_action_label(p.action),
                                                &e,
                                            );
                                            show_toast(&msg, ToastVariant::Error);
                                        }
                                    }
                                    in_flight.set(None);
                                    data.restart();
                                });
                            }
                            pending.set(None);
                        },
                    }
                }
            }
        }
    }
}

/// Pure helper — picks the (title, body, confirm-label) i18n triple for
/// the per-row ConfirmDialog on the agents admin page.
pub(crate) fn agent_dialog_copy(action: RowAction) -> (&'static str, &'static str, &'static str) {
    match action {
        RowAction::Approve => (
            "agents_admin.approve_confirm_title",
            "agents_admin.approve_confirm_body",
            "common.approve",
        ),
        RowAction::Suspend => (
            "agents_admin.suspend_confirm_title",
            "agents_admin.suspend_confirm_body",
            "common.suspend",
        ),
        RowAction::Revoke => (
            "agents_admin.revoke_confirm_title",
            "agents_admin.revoke_confirm_body",
            "common.revoke",
        ),
    }
}

/// Pure helper — short label for the per-action toast on the agents
/// page. Stays here so it's unit-testable.
pub(crate) fn agent_action_label(action: RowAction) -> &'static str {
    match action {
        RowAction::Approve => "agent approve",
        RowAction::Suspend => "agent suspend",
        RowAction::Revoke => "agent revoke",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_matches_action_polarity() {
        assert_eq!(agent_action_label(RowAction::Approve), "agent approve");
        assert_eq!(agent_action_label(RowAction::Suspend), "agent suspend");
        assert_eq!(agent_action_label(RowAction::Revoke), "agent revoke");
    }

    #[test]
    fn dialog_copy_picks_distinct_keys_per_action() {
        let (t1, _, _) = agent_dialog_copy(RowAction::Approve);
        let (t2, _, _) = agent_dialog_copy(RowAction::Suspend);
        let (t3, _, _) = agent_dialog_copy(RowAction::Revoke);
        assert!(t1.contains("approve"));
        assert!(t2.contains("suspend"));
        assert!(t3.contains("revoke"));
    }
}
