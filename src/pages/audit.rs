use arkret_wire::event_kind_str;
use dioxus::prelude::*;

use crate::api::audit;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, LabelFor};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::types::AdminAuditEntryExt;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[derive(Debug, Clone, Default, PartialEq)]
struct DraftFilter {
    action: String,
    actor_id: String,
    realm_id: String,
    since: String,
    until: String,
    /// AKP-0007 event kind filter. Empty = no filter.
    kind: String,
}

impl DraftFilter {
    fn to_query(&self) -> audit::AuditFilter {
        audit::AuditFilter {
            action: trim_to_option(&self.action),
            actor_id: trim_to_option(&self.actor_id),
            realm_id: trim_to_option(&self.realm_id),
            kind: trim_to_option(&self.kind),
            since: trim_to_option(&self.since),
            until: trim_to_option(&self.until),
        }
    }

    fn is_empty(&self) -> bool {
        self.action.trim().is_empty()
            && self.actor_id.trim().is_empty()
            && self.realm_id.trim().is_empty()
            && self.since.trim().is_empty()
            && self.until.trim().is_empty()
            && self.kind.trim().is_empty()
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
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut expanded = use_signal(|| None::<String>);
    let mut draft = use_signal(DraftFilter::default);
    let mut applied = use_signal(DraftFilter::default);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let applied_filter = applied.read().clone();

    let mut data = use_resource(move || {
        let filter = applied_filter.clone();
        let cursor = cursor_snapshot.clone();
        async move { audit::list_audit_entries(cursor.as_deref(), PAGE_SIZE, filter.to_query()).await }
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
                        LabelFor { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_action")} }
                        Input {
                            placeholder: t("audit.filter_action_placeholder"),
                            value: draft.read().action.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().action = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        LabelFor { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_actor")} }
                        Input {
                            placeholder: t("audit.filter_actor_placeholder"),
                            value: draft.read().actor_id.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().actor_id = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        LabelFor { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_realm")} }
                        Input {
                            placeholder: t("audit.filter_realm_placeholder"),
                            value: draft.read().realm_id.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().realm_id = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        LabelFor { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_since")} }
                        Input {
                            r#type: "datetime-local".to_string(),
                            value: draft.read().since.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().since = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        LabelFor { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_until")} }
                        Input {
                            r#type: "datetime-local".to_string(),
                            value: draft.read().until.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().until = evt.value();
                            },
                        }
                    }
                    // AKP-0007 event-kind filter dropdown. The values
                    // come straight from the SDK's generated event-kind
                    // registry, so a registry rename breaks this build.
                    div { class: "space-y-1",
                        LabelFor { class: "text-xs text-muted-foreground".to_string(), {t("audit.filter_event_kind")} }
                        select {
                            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                            value: draft.read().kind.clone(),
                            onchange: move |evt| {
                                draft.write().kind = evt.value();
                            },
                            option { value: "", "—" }
                            option { value: event_kind_str::CIRCLE_CREATE, {t("audit.filter_event_kind_circle_create")} }
                            option { value: event_kind_str::CIRCLE_UPDATE, {t("audit.filter_event_kind_circle_update")} }
                            option { value: event_kind_str::CIRCLE_ARCHIVE, {t("audit.filter_event_kind_circle_archive")} }
                            option { value: event_kind_str::CIRCLE_RESTORE, {t("audit.filter_event_kind_circle_restore")} }
                            option { value: event_kind_str::CIRCLE_TOMBSTONE, {t("audit.filter_event_kind_circle_tombstone")} }
                            option { value: event_kind_str::CIRCLE_MEMBER_STATE, {t("audit.filter_event_kind_circle_member_state")} }
                            option { value: event_kind_str::CAPABILITY_GRANT, {t("audit.filter_event_kind_capability_grant")} }
                            option { value: event_kind_str::CAPABILITY_REVOKE, {t("audit.filter_event_kind_capability_revoke")} }
                        }
                    }
                }
                div { class: "mt-4 flex flex-wrap items-center gap-2",
                    Button {
                        onclick: move |_| {
                            cursor_stack.set(vec![None::<String>]);
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
                            cursor_stack.set(vec![None::<String>]);
                        },
                        {t("audit.filter_reset")}
                    }
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => {
                    let next_cursor = resp.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    rsx! {
                        div { class: "rounded-md border",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead { {t("audit.id")} }
                                        TableHead { {t("audit.action")} }
                                        TableHead { {t("audit.actor_id")} }
                                        TableHead { {t("audit.realm_id")} }
                                        TableHead { {t("audit.outcome")} }
                                        // AKP-0008 — envelope attribution columns
                                        // (wire keys, surfaced from `payload`):
                                        // executed_by / authorization_ref /
                                        // actor_kind (reducer-stamped).
                                        TableHead { "executed_by" }
                                        TableHead { "authz_ref" }
                                        TableHead { "actor_kind" }
                                        TableHead { {t("audit.timestamp")} }
                                    }
                                }
                                TableBody {
                                    if resp.entries.is_empty() {
                                        TableRow {
                                            TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                {t("audit.no_entries")}
                                            }
                                        }
                                    } else {
                                        for entry in resp.entries.iter() {
                                            {
                                                let id = entry.id.clone();
                                                let action = entry.action.clone();
                                                let actor_id = entry.actor_id.clone().unwrap_or_else(|| "-".to_string());
                                                let realm_id = entry.realm_id.clone().unwrap_or_else(|| "-".to_string());
                                                let outcome = entry.outcome.clone().unwrap_or_else(|| "-".to_string());
                                                let timestamp = entry
                                                    .created_at
                                                    .map(|ts| ts.to_rfc3339())
                                                    .unwrap_or_else(|| "-".to_string());
                                                let executed_by = entry.executed_by().unwrap_or_else(|| "-".to_string());
                                                let authorization_ref = entry.authorization_ref().unwrap_or_else(|| "-".to_string());
                                                let actor_kind = entry.actor_kind().unwrap_or_else(|| "-".to_string());
                                                let payload = entry.payload.clone();
                                                let is_expanded = expanded.read().as_ref() == Some(&id);

                                                rsx! {
                                                    TableRow {
                                                        key: "{id}",
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
                                                        TableCell { class: "font-mono text-xs max-w-[160px] truncate".to_string(), "{realm_id}" }
                                                        TableCell { class: "text-xs".to_string(), "{outcome}" }
                                                        TableCell { class: "font-mono text-xs max-w-[160px] truncate".to_string(), "{executed_by}" }
                                                        TableCell { class: "font-mono text-xs max-w-[160px] truncate".to_string(), "{authorization_ref}" }
                                                        TableCell { class: "text-xs".to_string(), "{actor_kind}" }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{timestamp}" }
                                                    }
                                                    if is_expanded {
                                                        TableRow {
                                                            key: "{id}-details",
                                                            TableCell { colspan: 99, class: "p-0".to_string(),
                                                                div { class: "p-4 bg-muted/50",
                                                                    p { class: "text-xs font-medium mb-2", {t("audit.details")} }
                                                                    pre { class: "text-xs font-mono bg-muted p-3 rounded overflow-auto max-h-64",
                                                                        {payload.map(|d| serde_json::to_string_pretty(&d).unwrap_or_else(|_| "{}".to_string())).unwrap_or_else(|| t("audit.no_details"))}
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
