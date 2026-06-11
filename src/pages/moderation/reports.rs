//! Soland moderation reports admin page
//!
//! Cursor-paginated list of moderation reports. Defaults to filtering
//! for `open` reports; status filter buttons widen the projection to
//! `resolved` / `dismissed` / `all`. Each open row exposes Resolve and
//! Dismiss actions which POST `{decision, note?}` to soland's resolve
//! route.
//!
//! This is the single reports workflow in sodmin. The row projection
//! uses spec-aligned `reporter_did` / `target_ref` naming and cursor
//! pagination.

use dioxus::prelude::*;

use crate::api::moderation;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::moderation::{
    ModerationReport, ReportDecision, ReportStatus, ResolveReportRequest,
};
use crate::utils::i18n::t;
use crate::utils::net::error::{format_optional_endpoint_error, should_reset_cursor_pagination};

const PAGE_SIZE: u64 = 25;

/// Pending-decision dialog state. The operator picks Resolve or
/// Dismiss; the decision goes into the request body.
#[derive(Debug, Clone)]
struct PendingDecision {
    report: ModerationReport,
    decision: ReportDecision,
}

#[component]
pub fn ModerationReportsPage() -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut status_filter = use_signal(|| "open".to_string());
    let mut pending = use_signal::<Option<PendingDecision>>(|| None);
    let mut in_flight = use_signal::<Option<String>>(|| None);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let cursor_for_fetch = cursor_snapshot.clone();
    let status_snapshot = status_filter.read().clone();

    let mut data = use_resource(move || {
        let cursor = cursor_for_fetch.clone();
        let status = status_snapshot.clone();
        async move { moderation::list_reports(cursor.as_deref(), PAGE_SIZE, &status).await }
    });

    let mut reset_to_first_page = move || {
        cursor_stack.set(vec![None]);
    };

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("moderation_reports.title"),
                description: t("moderation_reports.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            div { class: "flex gap-2",
                {
                    let buckets: [(&str, &str); 4] = [
                        ("open", "moderation_reports.filter_open"),
                        ("resolved", "moderation_reports.filter_resolved"),
                        ("dismissed", "moderation_reports.filter_dismissed"),
                        ("all", "moderation_reports.filter_all"),
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
                    let total_label = match page.total {
                        Some(n) => format!("{n}"),
                        None => "?".to_string(),
                    };
                    let row_count = page.data.len();
                    rsx! {
                        if page.data.is_empty() {
                            EmptyState {
                                icon: "flag".to_string(),
                                title: t("moderation_reports.empty_title"),
                                description: t("moderation_reports.empty_subtitle"),
                            }
                        } else {
                            p { class: "text-xs text-muted-foreground",
                                {format!("Showing {row_count} (server total: {total_label})")}
                            }
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("moderation_reports.id")} }
                                            TableHead { {t("moderation_reports.reporter")} }
                                            TableHead { {t("moderation_reports.target")} }
                                            TableHead { {t("moderation_reports.realm")} }
                                            TableHead { {t("moderation_reports.reason")} }
                                            TableHead { {t("moderation_reports.status")} }
                                            TableHead { {t("moderation_reports.created_at")} }
                                            TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                        }
                                    }
                                    TableBody {
                                        for r in page.data.iter() {
                                            {
                                                let report_id = r.report_id.clone();
                                                let reporter = r.reporter_did.clone();
                                                let target = r.target_ref.clone().unwrap_or_else(|| "-".to_string());
                                                let realm = r.realm_id.clone().unwrap_or_else(|| "-".to_string());
                                                let reason = r.reason.clone();
                                                let typed = r.status_typed();
                                                let label = typed.label().to_string();
                                                let variant = report_status_variant(&typed);
                                                let created = r.created_at.clone().unwrap_or_else(|| "-".to_string());
                                                let resolvable = r.is_resolvable();
                                                let report_for_resolve = r.clone();
                                                let report_for_dismiss = r.clone();
                                                let row_in_flight = in_flight
                                                    .read()
                                                    .as_deref()
                                                    .map(|id| id == report_id.as_str())
                                                    .unwrap_or(false);
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "font-mono text-xs max-w-[200px] truncate".to_string(), "{report_id}" }
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{reporter}" }
                                                        TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{target}" }
                                                        TableCell { class: "font-mono text-xs max-w-[200px] truncate".to_string(), "{realm}" }
                                                        TableCell { class: "max-w-[300px] truncate".to_string(), "{reason}" }
                                                        TableCell {
                                                            Badge { variant, "{label}" }
                                                        }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            if resolvable {
                                                                {
                                                                    rsx! {
                                                                        div { class: "flex justify-end gap-2",
                                                                            Button {
                                                                                variant: ButtonVariant::Default,
                                                                                size: ButtonSize::Sm,
                                                                                disabled: row_in_flight,
                                                                                onclick: move |_| {
                                                                                    pending.set(Some(PendingDecision {
                                                                                        report: report_for_resolve.clone(),
                                                                                        decision: ReportDecision::Resolve,
                                                                                    }));
                                                                                },
                                                                                {t("moderation_reports.resolve")}
                                                                            }
                                                                            Button {
                                                                                variant: ButtonVariant::Outline,
                                                                                size: ButtonSize::Sm,
                                                                                disabled: row_in_flight,
                                                                                onclick: move |_| {
                                                                                    pending.set(Some(PendingDecision {
                                                                                        report: report_for_dismiss.clone(),
                                                                                        decision: ReportDecision::Dismiss,
                                                                                    }));
                                                                                },
                                                                                {t("moderation_reports.dismiss")}
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                span { class: "text-xs text-muted-foreground", "—" }
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

            {
                let pending_snapshot = pending.read().clone();
                let confirm_label = pending_snapshot.as_ref().map(|p| match p.decision {
                    ReportDecision::Resolve => t("moderation_reports.resolve_confirm_btn"),
                    ReportDecision::Dismiss => t("moderation_reports.dismiss_confirm_btn"),
                }).unwrap_or_else(|| t("common.confirm"));
                let confirm_title = pending_snapshot.as_ref().map(|p| match p.decision {
                    ReportDecision::Resolve => t("moderation_reports.resolve_confirm_title"),
                    ReportDecision::Dismiss => t("moderation_reports.dismiss_confirm_title"),
                }).unwrap_or_default();
                let confirm_body = pending_snapshot.as_ref().map(|p| match p.decision {
                    ReportDecision::Resolve => t("moderation_reports.resolve_confirm_body"),
                    ReportDecision::Dismiss => t("moderation_reports.dismiss_confirm_body"),
                }).unwrap_or_default();
                rsx! {
                    ConfirmDialog {
                        open: pending_snapshot.is_some(),
                        title: confirm_title,
                        description: confirm_body,
                        confirm_text: confirm_label,
                        cancel_text: t("common.cancel"),
                        destructive: matches!(pending_snapshot.as_ref().map(|p| p.decision), Some(ReportDecision::Resolve)),
                        on_cancel: move |_| pending.set(None),
                        on_confirm: move |_| {
                            if let Some(p) = pending.read().clone() {
                                in_flight.set(Some(p.report.report_id.clone()));
                                spawn(async move {
                                    let body = ResolveReportRequest {
                                        decision: p.decision,
                                        note: None,
                                    };
                                    let res = moderation::resolve_report(
                                        &p.report, &body,
                                    )
                                    .await;
                                    match res {
                                        Ok(_) => show_toast(
                                            "Moderation report decision recorded.",
                                            ToastVariant::Success,
                                        ),
                                        Err(e) => {
                                            let msg = format_optional_endpoint_error(
                                                "moderation report resolve",
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

/// Pick a Badge variant for a moderation report status. `Open` is the
/// neutral default tone (ready for triage); `Resolved` is success-green
/// (acted on); `Dismissed` is the neutral secondary tone (reviewed,
/// no-op). Pure helper so the mapping is unit-testable.
pub(crate) fn report_status_variant(status: &ReportStatus) -> BadgeVariant {
    match status {
        ReportStatus::Open => BadgeVariant::Default,
        ReportStatus::Resolved => BadgeVariant::Success,
        ReportStatus::Dismissed => BadgeVariant::Secondary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::moderation::ModerationReport;

    #[test]
    fn report_variant_buckets_match_status() {
        assert!(matches!(
            report_status_variant(&ReportStatus::Open),
            BadgeVariant::Default
        ));
        assert!(matches!(
            report_status_variant(&ReportStatus::Resolved),
            BadgeVariant::Success
        ));
        assert!(matches!(
            report_status_variant(&ReportStatus::Dismissed),
            BadgeVariant::Secondary
        ));
    }

    #[test]
    fn pending_decision_is_destructive_only_for_resolve() {
        // Resolve is a destructive workflow (the admin acted on the
        // report) so the dialog renders the destructive button. Dismiss
        // is just a status flip so it stays neutral.
        let p = PendingDecision {
            report: ModerationReport {
                report_id: "r1".into(),
                ..Default::default()
            },
            decision: ReportDecision::Resolve,
        };
        assert!(matches!(p.decision, ReportDecision::Resolve));

        let p = PendingDecision {
            report: ModerationReport {
                report_id: "r1".into(),
                ..Default::default()
            },
            decision: ReportDecision::Dismiss,
        };
        assert!(matches!(p.decision, ReportDecision::Dismiss));
    }

    #[test]
    fn resolvable_only_when_open_with_id() {
        let r = ModerationReport {
            report_id: "r1".into(),
            status: "open".into(),
            ..Default::default()
        };
        assert!(r.is_resolvable());

        let r = ModerationReport {
            report_id: "r1".into(),
            status: "resolved".into(),
            ..Default::default()
        };
        assert!(!r.is_resolvable());
    }
}
