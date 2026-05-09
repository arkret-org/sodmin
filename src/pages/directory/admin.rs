//! Directory admin (Round 26, F3 — full mutation flow).
//!
//! Cursor-paginated list of soland-side directory entries (public
//! discovery directory) with per-row Approve / Reject buttons.
//! ConfirmDialog destructive on Reject. 404-tolerant. Each click emits
//! a structured client-side audit line via
//! `utils::audit::emit_admin_audit`.
//!
//! Note: directory entries use Approve/Reject (publish/delist) rather
//! than the Suspend lifecycle that applets and agents have — a Pending
//! entry that's rejected is simply not listed; there's no "soft-revoke"
//! concept for a directory listing.

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
use crate::pages::applets::admin::approval_variant;
use coauth_admin_types::applets_admin::ApprovalActionRequest;
use crate::utils::audit::{AdminAuditOutcome, emit_admin_audit};
use crate::utils::error::format_optional_endpoint_error;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

/// Directory entries have a simpler two-action lifecycle than
/// applets/agents — there's no Suspend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectoryAction {
    Approve,
    Reject,
}

impl DirectoryAction {
    pub(crate) fn wire(&self) -> &'static str {
        match self {
            DirectoryAction::Approve => "approve",
            DirectoryAction::Reject => "reject",
        }
    }

    pub(crate) fn is_destructive(&self) -> bool {
        matches!(self, DirectoryAction::Reject)
    }
}

#[derive(Debug, Clone)]
struct PendingDecision {
    id: String,
    action: DirectoryAction,
}

#[component]
pub fn DirectoryAdminPage() -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut pending = use_signal::<Option<PendingDecision>>(|| None);
    let mut in_flight = use_signal::<Option<String>>(|| None);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);

    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        async move { admin_api::list_directory_admin(cursor.as_deref(), PAGE_SIZE).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("directory_admin.title"),
                description: t("directory_admin.subtitle"),
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
                                icon: "globe".to_string(),
                                title: t("directory_admin.empty_title"),
                                description: t("directory_admin.empty_subtitle"),
                            }
                        } else {
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("directory_admin.entry_id")} }
                                            TableHead { {t("directory_admin.kind")} }
                                            TableHead { {t("directory_admin.label")} }
                                            TableHead { {t("directory_admin.owner_did")} }
                                            TableHead { {t("directory_admin.status")} }
                                            TableHead { {t("directory_admin.published_at")} }
                                            TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                        }
                                    }
                                    TableBody {
                                        for r in page.data.iter() {
                                            {
                                                let id = r.entry_id.clone();
                                                let kind = r.kind.clone();
                                                let label_str = r.label.clone().unwrap_or_default();
                                                let owner = r.owner_did.clone().unwrap_or_else(|| "-".to_string());
                                                let typed = r.status_typed();
                                                let label = typed.label().to_string();
                                                let variant = approval_variant(&typed);
                                                let published = r.published_at.clone().unwrap_or_else(|| "-".to_string());
                                                let approvable = typed.is_approvable();
                                                let rejectable = typed.is_revocable();
                                                let id_a = id.clone();
                                                let id_r = id.clone();
                                                let row_busy = in_flight
                                                    .read()
                                                    .as_deref()
                                                    .map(|x| x == id.as_str())
                                                    .unwrap_or(false);
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{id}" }
                                                        TableCell { "{kind}" }
                                                        TableCell { "{label_str}" }
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{owner}" }
                                                        TableCell { Badge { variant, "{label}" } }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{published}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            div { class: "flex justify-end gap-2",
                                                                Button {
                                                                    variant: ButtonVariant::Default,
                                                                    size: ButtonSize::Sm,
                                                                    disabled: !approvable || row_busy,
                                                                    onclick: move |_| {
                                                                        pending.set(Some(PendingDecision {
                                                                            id: id_a.clone(),
                                                                            action: DirectoryAction::Approve,
                                                                        }));
                                                                    },
                                                                    {t("common.approve")}
                                                                }
                                                                Button {
                                                                    variant: ButtonVariant::Destructive,
                                                                    size: ButtonSize::Sm,
                                                                    disabled: !rejectable || row_busy,
                                                                    onclick: move |_| {
                                                                        pending.set(Some(PendingDecision {
                                                                            id: id_r.clone(),
                                                                            action: DirectoryAction::Reject,
                                                                        }));
                                                                    },
                                                                    {t("common.reject")}
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
                let action = p.as_ref().map(|x| x.action).unwrap_or(DirectoryAction::Approve);
                let (title_key, body_key, confirm_label) = directory_dialog_copy(action);
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
                                        DirectoryAction::Approve => {
                                            admin_api::approve_directory_entry(&p.id, &body).await
                                        }
                                        DirectoryAction::Reject => {
                                            admin_api::reject_directory_entry(&p.id, &body).await
                                        }
                                    };
                                    match res {
                                        Ok(_) => {
                                            emit_admin_audit(
                                                "directory",
                                                &p.id,
                                                p.action.wire(),
                                                AdminAuditOutcome::Accepted,
                                                None,
                                            );
                                            show_toast(
                                                "Directory decision recorded.",
                                                ToastVariant::Success,
                                            );
                                        }
                                        Err(e) => {
                                            emit_admin_audit(
                                                "directory",
                                                &p.id,
                                                p.action.wire(),
                                                AdminAuditOutcome::from_http_status(e.status),
                                                None,
                                            );
                                            let msg = format_optional_endpoint_error(
                                                directory_action_label(p.action),
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
/// the directory ConfirmDialog.
pub(crate) fn directory_dialog_copy(
    action: DirectoryAction,
) -> (&'static str, &'static str, &'static str) {
    match action {
        DirectoryAction::Approve => (
            "directory_admin.approve_confirm_title",
            "directory_admin.approve_confirm_body",
            "common.approve",
        ),
        DirectoryAction::Reject => (
            "directory_admin.reject_confirm_title",
            "directory_admin.reject_confirm_body",
            "common.reject",
        ),
    }
}

/// Pure helper — pick the action verb the toast/audit-log uses for a
/// directory decision. Stays here so it's unit-testable.
pub(crate) fn directory_action_label(action: DirectoryAction) -> &'static str {
    match action {
        DirectoryAction::Approve => "directory approve",
        DirectoryAction::Reject => "directory reject",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_matches_action_polarity() {
        assert_eq!(
            directory_action_label(DirectoryAction::Approve),
            "directory approve"
        );
        assert_eq!(
            directory_action_label(DirectoryAction::Reject),
            "directory reject"
        );
    }

    #[test]
    fn destructive_only_for_reject() {
        assert!(!DirectoryAction::Approve.is_destructive());
        assert!(DirectoryAction::Reject.is_destructive());
    }

    #[test]
    fn wire_verb_matches_action() {
        assert_eq!(DirectoryAction::Approve.wire(), "approve");
        assert_eq!(DirectoryAction::Reject.wire(), "reject");
    }

    #[test]
    fn dialog_copy_distinct_per_action() {
        let (t1, b1, c1) = directory_dialog_copy(DirectoryAction::Approve);
        let (t2, b2, c2) = directory_dialog_copy(DirectoryAction::Reject);
        assert_ne!(t1, t2);
        assert_ne!(b1, b2);
        assert_eq!(c1, "common.approve");
        assert_eq!(c2, "common.reject");
    }
}
