//! Applets admin
//!
//! Cursor-paginated list of soland-side applet registrations with
//! per-row Approve / Suspend / Revoke buttons. ConfirmDialog is
//! destructive on Suspend / Revoke but not on Approve. 404-tolerant —
//! surfaces "endpoint not yet wired" toast when soland hasn't shipped
//! the surface yet. Each click emits a structured client-side audit
//! line via `utils::audit::emit_admin_audit` (in addition to the
//! soland-side audit row that the HTTP endpoint writes itself).

use dioxus::prelude::*;

use crate::api::applets_agents_directory_admin as admin_api;
use crate::components::dangerous_action_dialog::{DangerousActionDialog, applet_phrase};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::utils::audit::{AdminAuditOutcome, emit_admin_audit};
use crate::utils::error::format_optional_endpoint_error;
use crate::utils::i18n::t;
use coauth_admin_types::applets_admin::{ApprovalActionRequest, ApprovalStatus};

const PAGE_SIZE: u64 = 25;

/// One of the three per-row mutation verbs. Drives both the HTTP call
/// dispatch and the ConfirmDialog copy / destructiveness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowAction {
    Approve,
    Suspend,
    Revoke,
}

impl RowAction {
    pub(crate) fn wire(&self) -> &'static str {
        match self {
            RowAction::Approve => "approve",
            RowAction::Suspend => "suspend",
            RowAction::Revoke => "revoke",
        }
    }

    pub(crate) fn is_destructive(&self) -> bool {
        matches!(self, RowAction::Suspend | RowAction::Revoke)
    }
}

#[derive(Debug, Clone)]
struct PendingDecision {
    id: String,
    /// Applet name as shown in the table — used by the
    /// DangerousActionDialog to compute the typed-phrase gate (first 6
    /// chars of the name) on suspend / revoke. `None` falls back to
    /// the applet id.
    name: Option<String>,
    action: RowAction,
}

#[component]
pub fn AppletAdminPage() -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut pending = use_signal::<Option<PendingDecision>>(|| None);
    let mut in_flight = use_signal::<Option<String>>(|| None);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);

    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        async move { admin_api::list_applets(cursor.as_deref(), PAGE_SIZE).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("applets_admin.title"),
                description: t("applets_admin.subtitle"),
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
                                icon: "plug".to_string(),
                                title: t("applets_admin.empty_title"),
                                description: t("applets_admin.empty_subtitle"),
                            }
                        } else {
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("applets_admin.id")} }
                                            TableHead { {t("applets_admin.name")} }
                                            TableHead { {t("applets_admin.owner_did")} }
                                            TableHead { {t("applets_admin.status")} }
                                            TableHead { {t("applets_admin.registered_at")} }
                                            TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                        }
                                    }
                                    TableBody {
                                        for r in page.data.iter() {
                                            {
                                                let id = r.id.clone();
                                                let name = r.name.clone().unwrap_or_else(|| "-".to_string());
                                                let raw_name = r.name.clone();
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
                                                let name_a = raw_name.clone();
                                                let name_s = raw_name.clone();
                                                let name_r = raw_name.clone();
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
                                                                            name: name_a.clone(),
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
                                                                            name: name_s.clone(),
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
                                                                            name: name_r.clone(),
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
                        errcode: e.body.as_ref().map(|b| b.errcode.clone()),
                        request_id: e.request_id.clone(),
                        retry_after_ms: e.retry_after_ms,
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }

            {
                let p = pending.read().clone();
                let action = p.as_ref().map(|x| x.action).unwrap_or(RowAction::Approve);
                let (title_key, body_key, confirm_label) = applet_dialog_copy(action);
                let pending_id = p.as_ref().map(|x| x.id.clone()).unwrap_or_default();
                let pending_name = p.as_ref().and_then(|x| x.name.clone());
                // First-6-chars-of-name (or id fallback) gate per B.7
                // spec — only fires on the destructive verbs so an
                // approve click stays a simple ConfirmDialog.
                let phrase = applet_phrase(pending_name.as_deref(), &pending_id, 6);
                let on_confirm = move |_| {
                    if let Some(p) = pending.read().clone() {
                        in_flight.set(Some(p.id.clone()));
                        spawn(async move {
                            let body = ApprovalActionRequest::default();
                            let res = match p.action {
                                RowAction::Approve => admin_api::approve_applet(&p.id, &body).await,
                                RowAction::Suspend => admin_api::suspend_applet(&p.id, &body).await,
                                RowAction::Revoke => admin_api::revoke_applet(&p.id, &body).await,
                            };
                            match res {
                                Ok(_) => {
                                    emit_admin_audit(
                                        "applet",
                                        &p.id,
                                        p.action.wire(),
                                        AdminAuditOutcome::Accepted,
                                        None,
                                    );
                                    show_toast(
                                        "Applet decision recorded.",
                                        ToastVariant::Success,
                                    );
                                }
                                Err(e) => {
                                    emit_admin_audit(
                                        "applet",
                                        &p.id,
                                        p.action.wire(),
                                        AdminAuditOutcome::from_http_status(e.status),
                                        None,
                                    );
                                    let msg = format_optional_endpoint_error(
                                        applet_action_label(p.action),
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
                };
                let on_cancel = move |_| pending.set(None);
                if action.is_destructive() {
                    rsx! {
                        DangerousActionDialog {
                            open: p.is_some(),
                            title: t(title_key),
                            description: t(body_key),
                            confirmation_phrase: phrase,
                            confirm_text: t(confirm_label),
                            cancel_text: t("common.cancel"),
                            on_cancel,
                            on_confirm,
                        }
                    }
                } else {
                    rsx! {
                        ConfirmDialog {
                            open: p.is_some(),
                            title: t(title_key),
                            description: t(body_key),
                            confirm_text: t(confirm_label),
                            cancel_text: t("common.cancel"),
                            destructive: false,
                            on_cancel,
                            on_confirm,
                        }
                    }
                }
            }
        }
    }
}

/// Pure helper — pick a Badge variant for an applet/agent/directory
/// approval status. Shared semantics across the F1/F2/F3
/// pages so the visual contract is identical.
pub(crate) fn approval_variant(s: &ApprovalStatus) -> BadgeVariant {
    match s {
        ApprovalStatus::Pending => BadgeVariant::Default,
        ApprovalStatus::Approved => BadgeVariant::Success,
        ApprovalStatus::Suspended => BadgeVariant::Outline,
        ApprovalStatus::Revoked => BadgeVariant::Destructive,
    }
}

/// Pure helper — picks the (title, body, confirm-label) i18n triple for
/// the per-row ConfirmDialog. Extracted so dialog copy stays
/// unit-testable independent of the rsx! tree.
pub(crate) fn applet_dialog_copy(action: RowAction) -> (&'static str, &'static str, &'static str) {
    match action {
        RowAction::Approve => (
            "applets_admin.approve_confirm_title",
            "applets_admin.approve_confirm_body",
            "common.approve",
        ),
        RowAction::Suspend => (
            "applets_admin.suspend_confirm_title",
            "applets_admin.suspend_confirm_body",
            "common.suspend",
        ),
        RowAction::Revoke => (
            "applets_admin.revoke_confirm_title",
            "applets_admin.revoke_confirm_body",
            "common.revoke",
        ),
    }
}

/// Pure helper — short verb the toast / 404-tolerant error formatter
/// uses on the applet admin page.
pub(crate) fn applet_action_label(action: RowAction) -> &'static str {
    match action {
        RowAction::Approve => "applet approve",
        RowAction::Suspend => "applet suspend",
        RowAction::Revoke => "applet revoke",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_variant_matches_severity() {
        assert!(matches!(
            approval_variant(&ApprovalStatus::Pending),
            BadgeVariant::Default
        ));
        assert!(matches!(
            approval_variant(&ApprovalStatus::Approved),
            BadgeVariant::Success
        ));
        assert!(matches!(
            approval_variant(&ApprovalStatus::Suspended),
            BadgeVariant::Outline
        ));
        assert!(matches!(
            approval_variant(&ApprovalStatus::Revoked),
            BadgeVariant::Destructive
        ));
    }

    #[test]
    fn pending_decision_carries_action_polarity() {
        let p = PendingDecision {
            id: "a1".into(),
            name: None,
            action: RowAction::Approve,
        };
        assert_eq!(p.action, RowAction::Approve);
        let p = PendingDecision {
            id: "a1".into(),
            name: Some("acme-bot".into()),
            action: RowAction::Suspend,
        };
        assert_eq!(p.action, RowAction::Suspend);
        let p = PendingDecision {
            id: "a1".into(),
            name: None,
            action: RowAction::Revoke,
        };
        assert_eq!(p.action, RowAction::Revoke);
    }

    #[test]
    fn destructive_only_for_suspend_and_revoke() {
        assert!(!RowAction::Approve.is_destructive());
        assert!(RowAction::Suspend.is_destructive());
        assert!(RowAction::Revoke.is_destructive());
    }

    #[test]
    fn wire_verb_matches_action() {
        assert_eq!(RowAction::Approve.wire(), "approve");
        assert_eq!(RowAction::Suspend.wire(), "suspend");
        assert_eq!(RowAction::Revoke.wire(), "revoke");
    }

    #[test]
    fn dialog_copy_distinct_per_action() {
        let (t1, b1, c1) = applet_dialog_copy(RowAction::Approve);
        let (t2, b2, c2) = applet_dialog_copy(RowAction::Suspend);
        let (t3, b3, c3) = applet_dialog_copy(RowAction::Revoke);
        // Each action picks its own copy
        assert_ne!(t1, t2);
        assert_ne!(t2, t3);
        assert_ne!(b1, b2);
        assert_ne!(b2, b3);
        assert_eq!(c1, "common.approve");
        assert_eq!(c2, "common.suspend");
        assert_eq!(c3, "common.revoke");
    }

    #[test]
    fn action_label_distinct_per_action() {
        assert_eq!(applet_action_label(RowAction::Approve), "applet approve");
        assert_eq!(applet_action_label(RowAction::Suspend), "applet suspend");
        assert_eq!(applet_action_label(RowAction::Revoke), "applet revoke");
    }
}
