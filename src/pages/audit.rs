use dioxus::prelude::*;

use crate::api::audit;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::router::Route;
use crate::types::AuditScopeKind;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[derive(Debug, Clone, Default, PartialEq)]
struct DraftFilter {
    action: String,
    actor_id: String,
    target_type: String,
    target_id: String,
    since: String,
    until: String,
    /// P3A.5 — CXP-0007 event kind filter. Empty = no filter.
    event_kind: String,
}

impl DraftFilter {
    fn to_query(&self) -> audit::AuditFilter {
        audit::AuditFilter {
            action: trim_to_option(&self.action),
            actor_id: trim_to_option(&self.actor_id),
            target_type: trim_to_option(&self.target_type),
            target_id: trim_to_option(&self.target_id),
            since: trim_to_option(&self.since),
            until: trim_to_option(&self.until),
            event_kind: trim_to_option(&self.event_kind),
            effective_scope: None,
        }
    }

    fn is_empty(&self) -> bool {
        self.action.trim().is_empty()
            && self.actor_id.trim().is_empty()
            && self.target_type.trim().is_empty()
            && self.target_id.trim().is_empty()
            && self.since.trim().is_empty()
            && self.until.trim().is_empty()
            && self.event_kind.trim().is_empty()
    }
}

fn trim_to_option(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[component]
pub fn AuditLog() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut expanded = use_signal(|| None::<String>);
    let mut draft = use_signal(DraftFilter::default);
    let mut applied = use_signal(DraftFilter::default);

    let page_val = *page.read();
    let applied_filter = applied.read().clone();

    let mut data = use_resource(move || {
        let filter = applied_filter.clone();
        async move { audit::list_audit_entries(page_val, PAGE_SIZE, filter.to_query()).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("audit.title"),
                description: t("audit.subtitle"),
            }

            div { class: "rounded-md border bg-card p-4",
                div { class: "grid gap-3 md:grid-cols-3",
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_action")} }
                        Input {
                            placeholder: t("audit.filter_action_placeholder"),
                            value: draft.read().action.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().action = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_actor")} }
                        Input {
                            placeholder: t("audit.filter_actor_placeholder"),
                            value: draft.read().actor_id.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().actor_id = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_target_type")} }
                        Input {
                            placeholder: t("audit.filter_target_type_placeholder"),
                            value: draft.read().target_type.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().target_type = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_target_id")} }
                        Input {
                            placeholder: t("audit.filter_target_id_placeholder"),
                            value: draft.read().target_id.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().target_id = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_since")} }
                        Input {
                            r#type: "datetime-local".to_string(),
                            value: draft.read().since.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().since = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_until")} }
                        Input {
                            r#type: "datetime-local".to_string(),
                            value: draft.read().until.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().until = evt.value();
                            },
                        }
                    }
                    // P3A.5 — CXP-0007 event-kind filter dropdown.
                    // The seven cx.circle.* event kinds match the
                    // SDK's event-kind registry exactly.
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_event_kind")} }
                        select {
                            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                            value: draft.read().event_kind.clone(),
                            onchange: move |evt| {
                                draft.write().event_kind = evt.value();
                            },
                            option { value: "", "—" }
                            option { value: "ck.circle.create", {t("audit.filter_event_kind_circle_create")} }
                            option { value: "ck.circle.update", {t("audit.filter_event_kind_circle_update")} }
                            option { value: "ck.circle.archive", {t("audit.filter_event_kind_circle_archive")} }
                            option { value: "ck.circle.tombstone", {t("audit.filter_event_kind_circle_tombstone")} }
                            option { value: "ck.circle.member.state", {t("audit.filter_event_kind_circle_member_state")} }
                            option { value: "cx.circle.capability.grant", {t("audit.filter_event_kind_circle_capability_grant")} }
                            option { value: "cx.circle.capability.revoke", {t("audit.filter_event_kind_circle_capability_revoke")} }
                        }
                    }
                }
                div { class: "mt-4 flex flex-wrap items-center gap-2",
                    Button {
                        onclick: move |_| {
                            page.set(1);
                            applied.set(draft.read().clone());
                        },
                        {t("audit.filter_apply")}
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        disabled: draft.read().is_empty() && applied.read().is_empty(),
                        onclick: move |_| {
                            draft.set(DraftFilter::default());
                            applied.set(DraftFilter::default());
                            page.set(1);
                        },
                        {t("audit.filter_reset")}
                    }
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("audit.id")} }
                                    TableHead { {t("audit.action")} }
                                    TableHead { {t("audit.actor_id")} }
                                    TableHead { {t("audit.target_type")} }
                                    TableHead { {t("audit.target_id")} }
                                    TableHead { {t("audit.effective_scope")} }
                                    // CXP-0008 — new envelope columns:
                                    // executed_by / authorization_ref /
                                    // actor_kind (reducer-stamped).
                                    TableHead { "executed_by" }
                                    TableHead { "authz_ref" }
                                    TableHead { "actor_kind" }
                                    TableHead { {t("audit.timestamp")} }
                                    TableHead { {t("audit.source_ip")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("audit.no_entries")}
                                        }
                                    }
                                } else {
                                    for entry in resp.data.iter() {
                                        {
                                            let id = entry.id.clone();
                                            let action = entry.action.clone();
                                            let actor_id = entry.actor_id.clone().unwrap_or_else(|| "-".to_string());
                                            let target_type = entry.target_type.clone().unwrap_or_else(|| "-".to_string());
                                            let target_id = entry.target_id.clone().unwrap_or_else(|| "-".to_string());
                                            let timestamp = entry.timestamp.clone().unwrap_or_else(|| "-".to_string());
                                            let source_ip = entry.source_ip.clone().unwrap_or_else(|| "-".to_string());
                                            let executed_by = entry.executed_by.clone().unwrap_or_else(|| "-".to_string());
                                            let authorization_ref = entry.authorization_ref.clone().unwrap_or_else(|| "-".to_string());
                                            let actor_kind = entry.actor_kind.clone().unwrap_or_else(|| "-".to_string());
                                            let details = entry.details.clone();
                                            let is_expanded = expanded.read().as_ref() == Some(&id);
                                            // P3A.5 — render the
                                            // effective scope (Realm
                                            // vs Circle) with a deep
                                            // link when it is a
                                            // Circle id.
                                            let scope_kind = entry.scope_kind();

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(),
                                                        button {
                                                            class: "text-left w-full cursor-pointer",
                                                            onclick: {
                                                                let id = id.clone();
                                                                move |_| {
                                                                    if expanded.read().as_ref() == Some(&id) {
                                                                        expanded.set(None);
                                                                    } else {
                                                                        expanded.set(Some(id.clone()));
                                                                    }
                                                                }
                                                            },
                                                            "{id}"
                                                        }
                                                    }
                                                    TableCell { "{action}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{actor_id}" }
                                                    TableCell { "{target_type}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{target_id}" }
                                                    TableCell { class: "font-mono text-xs".to_string(),
                                                        {render_effective_scope(&scope_kind)}
                                                    }
                                                    TableCell { class: "font-mono text-xs max-w-[160px] truncate".to_string(), "{executed_by}" }
                                                    TableCell { class: "font-mono text-xs max-w-[160px] truncate".to_string(), "{authorization_ref}" }
                                                    TableCell { class: "text-xs".to_string(), "{actor_kind}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{timestamp}" }
                                                    TableCell { "{source_ip}" }
                                                }
                                                if is_expanded {
                                                    TableRow {
                                                        TableCell { colspan: 99, class: "p-0".to_string(),
                                                            div { class: "p-4 bg-muted/50",
                                                                p { class: "text-xs font-medium mb-2", "Details" }
                                                                pre { class: "text-xs font-mono bg-muted p-3 rounded overflow-auto max-h-64",
                                                                    {details.map(|d| serde_json::to_string_pretty(&d).unwrap_or_else(|_| "{}".to_string())).unwrap_or_else(|| "No details".to_string())}
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
                    }

                    Pagination {
                        page: page_val,
                        total: resp.total,
                        per_page: PAGE_SIZE,
                        on_page_change: move |p| page.set(p),
                    }
                },
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

/// P3A.5 — render the audit entry's effective scope. Circle scopes
/// get a deep link into `/circles/:id`; Realm scopes render as plain
/// text (the Realm directory does not yet have a `/realms/:id` show
/// page, see `TODO(circle-rollout-P3A.5)` for the deferred follow-up).
fn render_effective_scope(kind: &AuditScopeKind) -> Element {
    match kind {
        AuditScopeKind::Circle(id) => {
            let id_owned = id.clone();
            let label = id.clone();
            rsx! {
                Link {
                    to: Route::CircleShow { circle_id: id_owned },
                    class: "text-primary hover:underline",
                    title: t("audit.scope_jump"),
                    {format!("{}: {}", t("audit.scope_circle"), label)}
                }
            }
        }
        AuditScopeKind::Realm(id) => rsx! {
            span { class: "text-muted-foreground",
                {format!("{}: {}", t("audit.scope_realm"), id)}
            }
        },
        AuditScopeKind::Unknown => rsx! { span { class: "text-muted-foreground", "-" } },
    }
}
