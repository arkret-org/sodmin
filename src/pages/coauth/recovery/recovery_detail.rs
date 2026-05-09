//! Recovery ticket detail page (Round 25, C2).
//!
//! Per-ticket status timeline + admin approve/reject/advance/cancel
//! buttons. Each destructive action goes through ConfirmDialog. The
//! page is 404-tolerant — surfaces "endpoint not yet wired" toasts.

use dioxus::prelude::*;

use crate::api::recovery_admin;
use crate::components::ui::badge::Badge;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use coauth_admin_types::recovery_admin::{RecoveryActionRequest, RecoveryTicketStatus};
use crate::utils::error::format_optional_endpoint_error;
use crate::utils::i18n::t;

use super::recovery_list::ticket_status_variant;

/// Which lifecycle action the operator picked. Drives the
/// ConfirmDialog copy and the underlying API call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryAction {
    Approve,
    Reject,
    Advance,
    Cancel,
}

impl RecoveryAction {
    pub(crate) fn label_key(&self) -> &'static str {
        match self {
            RecoveryAction::Approve => "recovery_detail.approve",
            RecoveryAction::Reject => "recovery_detail.reject",
            RecoveryAction::Advance => "recovery_detail.advance",
            RecoveryAction::Cancel => "recovery_detail.cancel",
        }
    }

    pub(crate) fn confirm_title_key(&self) -> &'static str {
        match self {
            RecoveryAction::Approve => "recovery_detail.approve_confirm_title",
            RecoveryAction::Reject => "recovery_detail.reject_confirm_title",
            RecoveryAction::Advance => "recovery_detail.advance_confirm_title",
            RecoveryAction::Cancel => "recovery_detail.cancel_confirm_title",
        }
    }

    pub(crate) fn confirm_body_key(&self) -> &'static str {
        match self {
            RecoveryAction::Approve => "recovery_detail.approve_confirm_body",
            RecoveryAction::Reject => "recovery_detail.reject_confirm_body",
            RecoveryAction::Advance => "recovery_detail.advance_confirm_body",
            RecoveryAction::Cancel => "recovery_detail.cancel_confirm_body",
        }
    }

    /// Reject + Cancel are destructive (terminal), Approve + Advance
    /// are forward-progress.
    pub(crate) fn is_destructive(&self) -> bool {
        matches!(self, RecoveryAction::Reject | RecoveryAction::Cancel)
    }

    /// True when this action is meaningful for the given status.
    pub(crate) fn is_allowed_for(&self, status: &RecoveryTicketStatus) -> bool {
        match self {
            RecoveryAction::Approve | RecoveryAction::Reject => status.is_approvable(),
            RecoveryAction::Advance => status.is_advanceable(),
            RecoveryAction::Cancel => status.is_cancellable(),
        }
    }

    /// Audit-toast verb shown when the API call succeeds.
    pub(crate) fn endpoint_label(&self) -> &'static str {
        match self {
            RecoveryAction::Approve => "recovery approve",
            RecoveryAction::Reject => "recovery reject",
            RecoveryAction::Advance => "recovery advance",
            RecoveryAction::Cancel => "recovery cancel",
        }
    }
}

#[component]
pub fn RecoveryTicketDetailPage(ticket_id: String) -> Element {
    let mut pending = use_signal::<Option<RecoveryAction>>(|| None);
    let mut in_flight = use_signal(|| false);

    let id_for_resource = ticket_id.clone();
    let mut data = use_resource(move || {
        let id = id_for_resource.clone();
        async move { recovery_admin::get_ticket(&id).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("recovery_detail.title"),
                description: t("recovery_detail.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            div { class: "text-sm",
                Link {
                    to: Route::RecoveryTicketList {},
                    class: "text-primary hover:underline".to_string(),
                    {t("recovery_detail.back_to_list")}
                }
            }

            match &*data.read() {
                Some(Ok(detail)) => {
                    let typed = detail.ticket.status_typed();
                    let label = typed.label().to_string();
                    let variant = ticket_status_variant(&typed);
                    let actions: [(RecoveryAction, ButtonVariant); 4] = [
                        (RecoveryAction::Approve, ButtonVariant::Default),
                        (RecoveryAction::Reject, ButtonVariant::Destructive),
                        (RecoveryAction::Advance, ButtonVariant::Default),
                        (RecoveryAction::Cancel, ButtonVariant::Outline),
                    ];
                    let timeline = detail.timeline.clone();
                    let ticket_id_meta = detail.ticket.ticket_id.clone();
                    let account_did = detail.ticket.account_did.clone();
                    let mode = detail.ticket.mode.clone().unwrap_or_else(|| "-".to_string());
                    let created = detail.ticket.created_at.clone().unwrap_or_else(|| "-".to_string());
                    let updated = detail.ticket.updated_at.clone().unwrap_or_else(|| "-".to_string());
                    let busy = *in_flight.read();
                    rsx! {
                        div { class: "rounded-md border p-4 space-y-2",
                            div { class: "flex items-center gap-3",
                                Badge { variant, "{label}" }
                                span { class: "font-mono text-xs text-muted-foreground",
                                    "{ticket_id_meta}"
                                }
                            }
                            div { class: "grid grid-cols-2 gap-2 text-sm",
                                div {
                                    strong { {t("recovery_list.account_did")} ":" }
                                    span { class: "font-mono ml-2", "{account_did}" }
                                }
                                div {
                                    strong { {t("recovery_list.mode")} ":" }
                                    span { class: "ml-2", "{mode}" }
                                }
                                div {
                                    strong { {t("recovery_list.created_at")} ":" }
                                    span { class: "ml-2", "{created}" }
                                }
                                div {
                                    strong { {t("recovery_detail.updated_at")} ":" }
                                    span { class: "ml-2", "{updated}" }
                                }
                            }
                        }

                        div { class: "flex flex-wrap gap-2",
                            for (action, btn_variant) in actions.iter() {
                                {
                                    let action_owned = *action;
                                    let allowed = action_owned.is_allowed_for(&typed);
                                    let label = t(action_owned.label_key());
                                    rsx! {
                                        Button {
                                            variant: btn_variant.clone(),
                                            size: ButtonSize::Sm,
                                            disabled: !allowed || busy,
                                            onclick: move |_| pending.set(Some(action_owned)),
                                            "{label}"
                                        }
                                    }
                                }
                            }
                            Link {
                                to: Route::RecoveryRestoreState { ticket_id: ticket_id_meta.clone() },
                                class: "inline-flex h-8 items-center rounded-md border px-2 text-xs font-medium transition-colors hover:bg-accent".to_string(),
                                {t("recovery_detail.open_restore_state")}
                            }
                            Link {
                                to: Route::RecoveryAuditLog { ticket_id: ticket_id_meta.clone() },
                                class: "inline-flex h-8 items-center rounded-md border px-2 text-xs font-medium transition-colors hover:bg-accent".to_string(),
                                {t("recovery_detail.open_audit_log")}
                            }
                        }

                        h2 { class: "text-lg font-semibold mt-4", {t("recovery_detail.timeline_title")} }
                        if timeline.is_empty() {
                            p { class: "text-sm text-muted-foreground", {t("recovery_detail.timeline_empty")} }
                        } else {
                            ol { class: "space-y-1 border-l pl-4",
                                for entry in timeline.iter() {
                                    {
                                        let at = entry.at.clone().unwrap_or_else(|| "-".to_string());
                                        let from = entry.from_status.clone().unwrap_or_else(|| "·".to_string());
                                        let to = entry.to_status.clone();
                                        let actor = entry.actor_did.clone().unwrap_or_default();
                                        let note = entry.note.clone().unwrap_or_default();
                                        rsx! {
                                            li { class: "text-sm",
                                                span { class: "font-mono text-xs text-muted-foreground", "{at}" }
                                                span { class: "ml-2", "{from} → {to}" }
                                                if !actor.is_empty() {
                                                    span { class: "ml-2 font-mono text-xs", "({actor})" }
                                                }
                                                if !note.is_empty() {
                                                    p { class: "text-xs text-muted-foreground ml-2", "{note}" }
                                                }
                                            }
                                        }
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
                let pending_snapshot = *pending.read();
                let title = pending_snapshot.map(|a| t(a.confirm_title_key())).unwrap_or_default();
                let body = pending_snapshot.map(|a| t(a.confirm_body_key())).unwrap_or_default();
                let confirm_text = pending_snapshot.map(|a| t(a.label_key())).unwrap_or_else(|| t("common.confirm"));
                let destructive = pending_snapshot.map(|a| a.is_destructive()).unwrap_or(false);
                let id = ticket_id.clone();
                rsx! {
                    ConfirmDialog {
                        open: pending_snapshot.is_some(),
                        title,
                        description: body,
                        confirm_text,
                        cancel_text: t("common.cancel"),
                        destructive,
                        on_cancel: move |_| pending.set(None),
                        on_confirm: move |_| {
                            if let Some(action) = *pending.read() {
                                let id = id.clone();
                                in_flight.set(true);
                                spawn(async move {
                                    let body = RecoveryActionRequest::default();
                                    let res = match action {
                                        RecoveryAction::Approve => recovery_admin::approve(&id, &body).await,
                                        RecoveryAction::Reject => recovery_admin::reject(&id, &body).await,
                                        RecoveryAction::Advance => recovery_admin::advance(&id, &body).await,
                                        RecoveryAction::Cancel => recovery_admin::cancel(&id, &body).await,
                                    };
                                    match res {
                                        Ok(_) => show_toast(
                                            "Recovery ticket action recorded.",
                                            ToastVariant::Success,
                                        ),
                                        Err(e) => {
                                            let msg = format_optional_endpoint_error(
                                                action.endpoint_label(),
                                                &e,
                                            );
                                            show_toast(&msg, ToastVariant::Error);
                                        }
                                    }
                                    in_flight.set(false);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approve_reject_only_allowed_when_pending() {
        assert!(RecoveryAction::Approve.is_allowed_for(&RecoveryTicketStatus::Pending));
        assert!(RecoveryAction::Reject.is_allowed_for(&RecoveryTicketStatus::Pending));
        assert!(!RecoveryAction::Approve.is_allowed_for(&RecoveryTicketStatus::Approved));
        assert!(!RecoveryAction::Reject.is_allowed_for(&RecoveryTicketStatus::Complete));
    }

    #[test]
    fn advance_only_allowed_after_approval() {
        assert!(RecoveryAction::Advance.is_allowed_for(&RecoveryTicketStatus::Approved));
        assert!(RecoveryAction::Advance.is_allowed_for(&RecoveryTicketStatus::ExecutorRunning));
        assert!(!RecoveryAction::Advance.is_allowed_for(&RecoveryTicketStatus::Pending));
        assert!(!RecoveryAction::Advance.is_allowed_for(&RecoveryTicketStatus::Complete));
    }

    #[test]
    fn destructive_actions_are_reject_and_cancel() {
        assert!(RecoveryAction::Reject.is_destructive());
        assert!(RecoveryAction::Cancel.is_destructive());
        assert!(!RecoveryAction::Approve.is_destructive());
        assert!(!RecoveryAction::Advance.is_destructive());
    }

    #[test]
    fn cancel_disabled_for_terminal_states() {
        assert!(!RecoveryAction::Cancel.is_allowed_for(&RecoveryTicketStatus::Complete));
        assert!(!RecoveryAction::Cancel.is_allowed_for(&RecoveryTicketStatus::Cancelled));
        assert!(RecoveryAction::Cancel.is_allowed_for(&RecoveryTicketStatus::Pending));
        assert!(RecoveryAction::Cancel.is_allowed_for(&RecoveryTicketStatus::Approved));
    }
}
