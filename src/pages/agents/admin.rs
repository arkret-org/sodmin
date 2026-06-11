//! Agents admin
//!
//! Cursor-paginated list of soland-side agent registrations with
//! per-row Approve / Suspend / Revoke buttons. ConfirmDialog destructive
//! on Suspend / Revoke. 404-tolerant. Each click emits a structured
//! client-side audit line via `utils::net::audit::emit_admin_audit`.
//!
//! Round 3 additions: row + header checkboxes for bulk revoke, a shared
//! `SearchInput` plumbed through `filter[name_or_id]` with a
//! client-side fallback, and the auto-refresh picker.

use std::collections::HashSet;

use coauth_admin_types::applets_admin::ApprovalActionRequestBody as ApprovalActionRequest;
use dioxus::prelude::*;

use crate::api::applets_agents_directory as admin_api;
use crate::components::dangerous_action_dialog::DangerousActionDialog;
use crate::components::ui::auto_refresh::{self, AutoRefreshPicker, RefreshInterval};
use crate::components::ui::badge::Badge;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::checkbox::{self, Checkbox};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::SearchInput;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::pages::applets::admin::{RowAction, approval_variant};
use crate::utils::fmt::search::matches_name_or_id;
use crate::utils::futures::join_all;
use crate::utils::i18n::t;
use crate::utils::net::audit::{AdminAuditOutcome, emit_admin_audit_server};
use crate::utils::net::error::format_optional_endpoint_error;

const PAGE_SIZE: u64 = 25;
const BULK_CONCURRENCY: usize = 5;
const AUTOREFRESH_STORAGE_KEY: &str = "sodmin.agents.autorefresh";

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
    let mut search = use_signal(String::new);
    let mut selected = use_signal::<HashSet<String>>(HashSet::new);
    let mut show_bulk_dialog = use_signal(|| false);
    let mut bulk_progress = use_signal::<Option<(usize, usize)>>(|| None);
    let mut autorefresh = use_signal(|| auto_refresh::load(AUTOREFRESH_STORAGE_KEY));

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);

    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        async move { admin_api::list_agents_admin(cursor.as_deref(), PAGE_SIZE).await }
    });

    let mut interval_handle = use_signal::<Option<gloo_timers::callback::Interval>>(|| None);
    use_effect(move || {
        let choice = *autorefresh.read();
        interval_handle.set(None);
        if let Some(ms) = choice.millis() {
            let mut data = data;
            let handle = gloo_timers::callback::Interval::new(ms, move || {
                data.restart();
            });
            interval_handle.set(Some(handle));
        }
    });

    let search_val = search.read().clone();
    let visible_ids: Vec<String> = match &*data.read() {
        Some(Ok(page)) => page
            .data
            .iter()
            .filter(|r| matches_name_or_id(&search_val, &r.id, r.name.as_deref()))
            .filter(|r| r.status_typed().is_revocable())
            .map(|r| r.id.clone())
            .collect(),
        _ => Vec::new(),
    };
    let selected_on_page = visible_ids
        .iter()
        .filter(|id| selected.read().contains(id.as_str()))
        .count();
    let (header_checked, header_indeterminate) =
        checkbox::header_state(selected_on_page, visible_ids.len());
    let selected_count = selected.read().len();

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

            div { class: "flex items-center gap-4 flex-wrap",
                div { class: "flex-1 min-w-[240px]",
                    SearchInput {
                        placeholder: t("agents_admin.name").to_string() + " / " + t("agents_admin.id").as_str(),
                        value: search.read().clone(),
                        oninput: move |evt: FormEvent| {
                            search.set(evt.value());
                            cursor_stack.set(vec![None::<String>]);
                        },
                    }
                }
                AutoRefreshPicker {
                    storage_key: AUTOREFRESH_STORAGE_KEY.to_string(),
                    value: *autorefresh.read(),
                    onchange: move |v: RefreshInterval| autorefresh.set(v),
                }
            }

            if selected_count > 0 {
                div { class: "flex items-center justify-between rounded-md border bg-accent/40 px-3 py-2",
                    div { class: "text-sm text-foreground",
                        "{selected_count} selected"
                    }
                    div { class: "flex items-center gap-2",
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| selected.set(HashSet::new()),
                            "Clear"
                        }
                        Button {
                            variant: ButtonVariant::Destructive,
                            size: ButtonSize::Sm,
                            onclick: move |_| show_bulk_dialog.set(true),
                            {format!("Bulk Revoke ({} selected)", selected_count)}
                        }
                    }
                }
            }

            match &*data.read() {
                Some(Ok(page)) => {
                    let next_cursor = page.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    let visible_set: HashSet<String> = visible_ids.iter().cloned().collect();
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
                                            TableHead {
                                                class: "w-10".to_string(),
                                                Checkbox {
                                                    id: "agents-select-all".to_string(),
                                                    aria_label: "Select all revocable rows".to_string(),
                                                    checked: header_checked,
                                                    indeterminate: header_indeterminate,
                                                    onchange: move |_| {
                                                        let mut cur = selected.read().clone();
                                                        let all_selected = visible_set
                                                            .iter()
                                                            .all(|id| cur.contains(id));
                                                        if all_selected {
                                                            for id in visible_set.iter() {
                                                                cur.remove(id);
                                                            }
                                                        } else {
                                                            for id in visible_set.iter() {
                                                                cur.insert(id.clone());
                                                            }
                                                        }
                                                        selected.set(cur);
                                                    },
                                                }
                                            }
                                            TableHead { {t("agents_admin.id")} }
                                            TableHead { {t("agents_admin.name")} }
                                            TableHead { {t("agents_admin.owner_did")} }
                                            TableHead { {t("agents_admin.status")} }
                                            TableHead { {t("agents_admin.registered_at")} }
                                            TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                        }
                                    }
                                    TableBody {
                                        for r in page.data.iter().filter(|r| matches_name_or_id(&search_val, &r.id, r.name.as_deref())) {
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
                                                let id_for_check = id.clone();
                                                let is_checked = selected.read().contains(&id);
                                                let check_id = format!("agent-check-{}", id);
                                                let row_busy = in_flight
                                                    .read()
                                                    .as_deref()
                                                    .map(|x| x == id.as_str())
                                                    .unwrap_or(false);
                                                rsx! {
                                                    TableRow {
                                                        TableCell { class: "w-10".to_string(),
                                                            if revocable {
                                                                Checkbox {
                                                                    id: check_id,
                                                                    aria_label: format!("Select agent {}", id),
                                                                    checked: is_checked,
                                                                    onchange: move |_| {
                                                                        let mut cur = selected.read().clone();
                                                                        if cur.contains(&id_for_check) {
                                                                            cur.remove(&id_for_check);
                                                                        } else {
                                                                            cur.insert(id_for_check.clone());
                                                                        }
                                                                        selected.set(cur);
                                                                    },
                                                                }
                                                            }
                                                        }
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

            if let Some((done, total)) = *bulk_progress.read() {
                div { class: "rounded-md border bg-muted/40 px-3 py-2 text-sm",
                    "Bulk revoke in progress: {done} / {total}"
                }
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
                                            emit_admin_audit_server(
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
                                            emit_admin_audit_server(
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

            // Bulk-revoke confirmation. Uses the literal phrase REVOKE
            // as the typed gate since multi-row gates can't reuse the
            // last-4-of-id pattern.
            {
                let open = *show_bulk_dialog.read();
                let count = selected_count;
                rsx! {
                    DangerousActionDialog {
                        open,
                        title: format!("Bulk revoke {} agents?", count),
                        description: "All selected agents will be revoked. Each failure is reported individually; successful revokes are not rolled back.".to_string(),
                        confirmation_phrase: "REVOKE".to_string(),
                        confirm_text: "Bulk Revoke".to_string(),
                        cancel_text: t("common.cancel"),
                        on_cancel: move |_| show_bulk_dialog.set(false),
                        on_confirm: move |_| {
                            let ids: Vec<String> = selected.read().iter().cloned().collect();
                            show_bulk_dialog.set(false);
                            if ids.is_empty() {
                                return;
                            }
                            bulk_progress.set(Some((0, ids.len())));
                            spawn(async move {
                                let total = ids.len();
                                let (ok, failed) = run_bulk_agent_revoke(ids, BULK_CONCURRENCY, move |done| {
                                    bulk_progress.set(Some((done, total)));
                                })
                                .await;
                                bulk_progress.set(None);
                                selected.set(HashSet::new());
                                data.restart();
                                if !failed.is_empty() {
                                    let preview: Vec<String> = failed
                                        .iter()
                                        .take(3)
                                        .map(|(id, msg)| format!("{}: {}", id, msg))
                                        .collect();
                                    let suffix = if failed.len() > 3 {
                                        format!(" (+{} more)", failed.len() - 3)
                                    } else {
                                        String::new()
                                    };
                                    show_toast(
                                        &format!(
                                            "Revoked {} / {}. Failed: {}{}",
                                            ok,
                                            total,
                                            preview.join("; "),
                                            suffix
                                        ),
                                        ToastVariant::Error,
                                    );
                                } else {
                                    show_toast(
                                        &format!("Revoked {} agent(s)", ok),
                                        ToastVariant::Success,
                                    );
                                }
                            });
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

/// Bounded-concurrency bulk revoke for agents. Same shape as the
/// devices helper but calls `admin_api::revoke_agent` for each id.
async fn run_bulk_agent_revoke<F>(
    ids: Vec<String>,
    concurrency: usize,
    mut on_progress: F,
) -> (usize, Vec<(String, String)>)
where
    F: FnMut(usize) + 'static,
{
    let mut ok = 0usize;
    let mut failed: Vec<(String, String)> = Vec::new();
    let mut done = 0usize;
    let chunk = concurrency.max(1);
    for batch in ids.chunks(chunk) {
        let futures: Vec<_> = batch
            .iter()
            .cloned()
            .map(|id| async move {
                let body = ApprovalActionRequest::default();
                let res = admin_api::revoke_agent(&id, &body).await;
                (id, res)
            })
            .collect();
        let results = join_all(futures).await;
        for (id, res) in results {
            done += 1;
            match res {
                Ok(_) => ok += 1,
                Err(e) => failed.push((id, e.message)),
            }
        }
        on_progress(done);
    }
    (ok, failed)
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
        let (t1, ..) = agent_dialog_copy(RowAction::Approve);
        let (t2, ..) = agent_dialog_copy(RowAction::Suspend);
        let (t3, ..) = agent_dialog_copy(RowAction::Revoke);
        assert!(t1.contains("approve"));
        assert!(t2.contains("suspend"));
        assert!(t3.contains("revoke"));
    }
}
