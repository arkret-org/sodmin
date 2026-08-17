use dioxus::prelude::*;

use crate::api::policy;
use crate::components::dangerous_action_dialog::{DangerousActionDialog, confirmation_suffix};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::modal::{DialogActions, Modal};
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::policy::{
    AdminPolicy, CreatePolicyRequest, PolicyAuditEntry, PolicyEvidenceItem,
};
use crate::utils::i18n::t;
use crate::utils::net::error::{HttpError, should_reset_cursor_pagination};

const PAGE_SIZE: u64 = 25;

#[component]
pub fn PolicyList() -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let mut show_dialog = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| None::<String>);
    let mut editing_id = use_signal(|| None::<String>);
    let mut name = use_signal(String::new);
    let mut policy_kind = use_signal(String::new);
    let mut scope = use_signal(String::new);
    let mut subject_ref = use_signal(|| "*".to_string());
    let mut is_enabled = use_signal(|| true);
    let mut priority = use_signal(|| 0i32);
    let mut dialog_loading = use_signal(|| false);
    let mut selected_policy = use_signal(|| None::<AdminPolicy>);
    let mut dialog_error = use_signal(|| None::<String>);
    let mut dialog_read_only = use_signal(|| false);
    let mut page_error = use_signal(|| None::<String>);

    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let cursor_for_fetch = cursor_snapshot.clone();

    let mut data = use_resource(move || {
        let cursor = cursor_for_fetch.clone();
        async move { policy::list_policies(cursor.as_deref(), PAGE_SIZE).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("policy.title"),
                description: t("policy.subtitle"),
                Button {
                    variant: ButtonVariant::Default,
                    onclick: move |_| {
                        editing_id.set(None);
                        name.set(String::new());
                        policy_kind.set(String::new());
                        scope.set(String::new());
                        subject_ref.set("*".to_string());
                        is_enabled.set(true);
                        priority.set(0);
                        selected_policy.set(None);
                        dialog_error.set(None);
                        dialog_read_only.set(false);
                        show_dialog.set(true);
                    },
                    {t("common.create")}
                }
            }

            if let Some(message) = page_error.read().clone() {
                ErrorBanner { message }
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
                                        TableHead { {t("policy.id")} }
                                        TableHead { {t("policy.name")} }
                                        TableHead { {t("policy.policy_kind")} }
                                        TableHead { {t("policy.scope")} }
                                        TableHead { "Subject" }
                                        TableHead { {t("policy.enabled")} }
                                        TableHead { {t("policy.priority")} }
                                        TableHead { {t("policy.guardrails")} }
                                        TableHead { {t("policy.updated_at")} }
                                        TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                    }
                                }
                                TableBody {
                                    if resp.data.is_empty() {
                                        TableRow {
                                            TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                {t("policy.no_policies")}
                                            }
                                        }
                                    } else {
                                        for p in resp.data.iter() {
                                            {
                                                let id = p.id.clone();
                                                let p_name = p.name.clone();
                                                let p_type = p.policy_kind.clone().unwrap_or_else(|| "-".to_string());
                                                let p_scope = p.scope.clone().unwrap_or_else(|| "-".to_string());
                                                let p_scope_for_edit = p.scope.clone().unwrap_or_default();
                                                let p_subject = p.subject_ref.clone().unwrap_or_else(|| "-".to_string());
                                                let p_subject_for_edit = p.subject_ref.clone().unwrap_or_else(|| "*".to_string());
                                                let p_enabled = p.is_enabled;
                                                let p_priority = p.priority;
                                                let updated = p.updated_at.clone().unwrap_or_else(|| "-".to_string());
                                                let guardrail_policy = p.clone();
                                                let read_only = p.safety.read_only;

                                                let id_for_edit = id.clone();
                                                let id_for_delete = id.clone();
                                                let policy_for_edit = p.clone();

                                                rsx! {
                                                    TableRow {
                                                        key: "{id}",
                                                        TableCell { class: "font-medium".to_string(), "{id}" }
                                                        TableCell { "{p_name}" }
                                                        TableCell { "{p_type}" }
                                                        TableCell { class: "max-w-[200px] truncate".to_string(), "{p_scope}" }
                                                        TableCell { class: "max-w-[200px] truncate font-mono text-xs".to_string(), "{p_subject}" }
                                                        TableCell {
                                                            if p_enabled {
                                                                Badge { variant: BadgeVariant::Success, {t("common.enabled")} }
                                                            } else {
                                                                Badge { variant: BadgeVariant::Default, {t("common.disabled")} }
                                                            }
                                                        }
                                                        TableCell { "{p_priority}" }
                                                        TableCell {
                                                            PolicyGuardrailBadges { policy: guardrail_policy }
                                                        }
                                                        TableCell { class: "text-muted-foreground".to_string(), "{updated}" }
                                                        TableCell { class: "text-right".to_string(),
                                                            div { class: "flex items-center justify-end gap-1",
                                                                Button {
                                                                    variant: ButtonVariant::Ghost,
                                                                    onclick: {
                                                                        let id = id_for_edit.clone();
                                                                        let n = p_name.clone();
                                                                        let t = policy_for_edit.policy_kind.clone().unwrap_or_default();
                                                                        let s = p_scope_for_edit.clone();
                                                                        let sr = p_subject_for_edit.clone();
                                                                        let e = p_enabled;
                                                                        let pr = p_priority;
                                                                        let policy = policy_for_edit.clone();
                                                                        move |_| {
                                                                            editing_id.set(Some(id.clone()));
                                                                            name.set(n.clone());
                                                                            policy_kind.set(t.clone());
                                                                            scope.set(s.clone());
                                                                            subject_ref.set(sr.clone());
                                                                            is_enabled.set(e);
                                                                            priority.set(pr);
                                                                            selected_policy.set(Some(policy.clone()));
                                                                            dialog_error.set(None);
                                                                            dialog_read_only.set(read_only);
                                                                            show_dialog.set(true);
                                                                        }
                                                                    },
                                                                    {if read_only { t("common.view") } else { t("common.edit") }}
                                                                }
                                                                Button {
                                                                    variant: ButtonVariant::Ghost,
                                                                    disabled: read_only,
                                                                    onclick: {
                                                                        let id = id_for_delete.clone();
                                                                        move |_| show_delete_dialog.set(Some(id.clone()))
                                                                    },
                                                                    {t("common.delete")}
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
        }

        Modal {
            open: *show_dialog.read(),
            title: if *dialog_read_only.read() {
                t("policy.pin_readonly_title")
            } else if editing_id.read().is_some() {
                t("policy.edit")
            } else {
                t("policy.create")
            },
            on_close: move |_| show_dialog.set(false),
            div { class: "space-y-3",
                if *dialog_read_only.read() {
                    div {
                        class: "rounded-md border bg-muted/40 px-3 py-2 text-sm text-muted-foreground",
                        role: "note",
                        {t("policy.pin_standard_unavailable_detail")}
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "pol-name".to_string(), {t("policy.name")} }
                    Input {
                        value: name.read().clone(),
                        disabled: *dialog_read_only.read(),
                        oninput: move |evt: FormEvent| name.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "pol-type".to_string(), {t("policy.policy_kind")} }
                    Input {
                        value: policy_kind.read().clone(),
                        disabled: *dialog_read_only.read(),
                        oninput: move |evt: FormEvent| policy_kind.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "pol-scope".to_string(), {t("policy.scope")} }
                    Input {
                        value: scope.read().clone(),
                        disabled: *dialog_read_only.read(),
                        oninput: move |evt: FormEvent| scope.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "pol-subject".to_string(), "Subject" }
                    Input {
                        value: subject_ref.read().clone(),
                        disabled: *dialog_read_only.read(),
                        oninput: move |evt: FormEvent| subject_ref.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "pol-priority".to_string(), {t("policy.priority")} }
                    Input {
                        r#type: "number".to_string(),
                        value: priority.read().to_string(),
                        disabled: *dialog_read_only.read(),
                        oninput: move |evt: FormEvent| {
                            if let Ok(v) = evt.value().parse() {
                                priority.set(v);
                            }
                        },
                    }
                }
                div { class: "flex items-center gap-2",
                    input {
                        r#type: "checkbox",
                        checked: *is_enabled.read(),
                        disabled: *dialog_read_only.read(),
                        onchange: move |evt: Event<FormData>| is_enabled.set(evt.checked()),
                    }
                    Label { {t("policy.enabled")} }
                }
                if let Some(message) = dialog_error.read().clone() {
                    ErrorBanner { message }
                }
                if let Some(policy) = selected_policy.read().clone() {
                    PolicyGuardrailPanel { policy }
                }
                if let Some(policy) = selected_policy.read().clone() {
                    if policy.safety.pin_summary.is_some() {
                        PolicyPinSafetyPanel { policy }
                    }
                }
            }
            if *dialog_read_only.read() {
                div { class: "flex justify-end pt-2",
                    Button {
                        variant: ButtonVariant::Default,
                        onclick: move |_| show_dialog.set(false),
                        {t("common.close")}
                    }
                }
            } else {
                DialogActions {
                    confirm_text: t("common.save"),
                    cancel_text: t("common.cancel"),
                    confirm_loading: *dialog_loading.read(),
                    on_cancel: move |_| show_dialog.set(false),
                    on_confirm: move |_| {
                        dialog_loading.set(true);
                        let req = CreatePolicyRequest {
                            name: name.read().clone(),
                            policy_kind: if policy_kind.read().is_empty() { None } else { Some(policy_kind.read().clone()) },
                            scope: if scope.read().is_empty() { None } else { Some(scope.read().clone()) },
                            subject_ref: if subject_ref.read().is_empty() { None } else { Some(subject_ref.read().clone()) },
                            rules: selected_policy.read().as_ref().and_then(|policy| policy.rules.clone()),
                            is_enabled: *is_enabled.read(),
                            priority: *priority.read(),
                            ..Default::default()
                        };
                        if policy::request_targets_pin_policy(&req) {
                            let message = t("policy.pin_edit_unavailable");
                            dialog_error.set(Some(message.clone()));
                            show_toast(&format!("Failed: {message}"), ToastVariant::Error);
                            dialog_loading.set(false);
                            return;
                        }
                        let edit = editing_id.read().clone();
                        spawn(async move {
                            let result = match edit {
                                Some(ref id) => policy::update_policy(id, &req).await.map(|_| ()),
                                None => policy::create_policy(&req).await.map(|_| ()),
                            };
                            match result {
                                Ok(_) => {
                                    show_toast(
                                        if edit.is_some() { "Policy updated" } else { "Policy created" },
                                        ToastVariant::Success,
                                    );
                                    dialog_error.set(None);
                                    show_dialog.set(false);
                                    data.restart();
                                }
                                Err(e) => {
                                    let message = policy_mutation_error_message(&e);
                                    dialog_error.set(Some(message.clone()));
                                    show_toast(&format!("Failed: {message}"), ToastVariant::Error);
                                }
                            }
                            dialog_loading.set(false);
                        });
                    },
                }
            }
        }

        DangerousActionDialog {
            open: show_delete_dialog.read().is_some(),
            title: t("common.delete"),
            description: "Are you sure you want to delete this policy?".to_string(),
            confirmation_phrase: confirmation_suffix(show_delete_dialog.read().as_deref().unwrap_or(""), 4),
            confirm_text: t("common.delete"),
            cancel_text: t("common.cancel"),
            on_confirm: move |_| {
                if let Some(id) = show_delete_dialog.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match policy::delete_policy(&id).await {
                            Ok(_) => {
                                page_error.set(None);
                                show_toast(&t("policy.toast_deleted"), ToastVariant::Success);
                                data.restart();
                            }
                            Err(e) => {
                                let message = policy_mutation_error_message(&e);
                                page_error.set(Some(message.clone()));
                                show_toast(&format!("Failed: {message}"), ToastVariant::Error);
                            }
                        }
                    });
                }
                show_delete_dialog.set(None);
            },
            on_cancel: move |_| show_delete_dialog.set(None),
        }
    }
}

#[component]
fn PolicyGuardrailBadges(policy: AdminPolicy) -> Element {
    let evidence_count = policy.guardrails.approval_evidence.len();
    let audit_count = policy.guardrails.audit_trail.len();
    let obligation_count = policy.guardrails.obligations.len();
    let has_required_scope = policy.guardrails.required_scope.is_some();
    let is_pin_policy = policy.safety.pin_summary.is_some();
    let is_read_only = policy.safety.read_only;
    let has_any = evidence_count > 0
        || audit_count > 0
        || obligation_count > 0
        || has_required_scope
        || is_pin_policy
        || is_read_only;
    let approval_label = t("policy.approval_evidence");
    let audit_label = t("policy.audit_events");
    let obligation_label = t("policy.obligations");

    rsx! {
        div { class: "flex flex-wrap items-center gap-1",
            if evidence_count > 0 {
                Badge { variant: BadgeVariant::Secondary, "{evidence_count} {approval_label}" }
            }
            if audit_count > 0 {
                Badge { variant: BadgeVariant::Outline, "{audit_count} {audit_label}" }
            }
            if obligation_count > 0 {
                Badge { variant: BadgeVariant::Default, "{obligation_count} {obligation_label}" }
            }
            if has_required_scope {
                Badge { variant: BadgeVariant::Destructive, {t("policy.permission_required")} }
            }
            if is_pin_policy {
                Badge { variant: BadgeVariant::Secondary, {t("policy.pin_policy")} }
            }
            if is_read_only {
                Badge { variant: BadgeVariant::Outline, {t("policy.read_only")} }
            }
            if !has_any {
                span { class: "text-xs text-muted-foreground", {t("policy.guardrails_none")} }
            }
        }
    }
}

#[component]
fn PolicyPinSafetyPanel(policy: AdminPolicy) -> Element {
    let safety = policy.safety.clone();
    let Some(summary) = safety.pin_summary else {
        return rsx! {};
    };
    let actions = summary.actions;
    let pin_scopes = summary.pin_scopes;
    let quota_limits = summary.quota_limits;
    let note_policy = summary
        .note_plaintext_policy
        .unwrap_or_else(|| t("policy.pin_no_public_summary"));
    let redacted = safety.redacted_private_categories;

    rsx! {
        div {
            class: "space-y-3 rounded-md border border-border bg-muted/30 p-3",
            "data-testid": "pin-policy-safety-panel",
            div { class: "flex flex-wrap items-center justify-between gap-2",
                h3 { class: "text-sm font-semibold", {t("policy.pin_safety_title")} }
                Badge { variant: BadgeVariant::Outline, {t("policy.pin_standard_unavailable")} }
            }
            p { class: "text-sm text-muted-foreground",
                {t("policy.pin_standard_unavailable_detail")}
            }
            if !redacted.is_empty() {
                div { class: "rounded-md border border-destructive/30 bg-destructive/5 p-2 text-sm",
                    div { class: "font-medium text-destructive", {t("policy.pin_redacted_private")} }
                    div { class: "mt-2 flex flex-wrap gap-1",
                        for category in redacted {
                            Badge { variant: BadgeVariant::Destructive, "{category}" }
                        }
                    }
                }
            }
            div { class: "grid gap-3 md:grid-cols-2",
                PinSummaryList {
                    label: t("policy.pin_actions"),
                    values: actions,
                }
                PinSummaryList {
                    label: t("policy.pin_scopes"),
                    values: pin_scopes,
                }
                PinSummaryList {
                    label: t("policy.pin_quota"),
                    values: quota_limits,
                }
                div { class: "space-y-1",
                    div { class: "text-xs font-semibold uppercase text-muted-foreground",
                        {t("policy.pin_note_policy")}
                    }
                    div { class: "text-sm font-mono break-all", "{note_policy}" }
                }
            }
        }
    }
}

#[component]
fn PinSummaryList(label: String, values: Vec<String>) -> Element {
    rsx! {
        div { class: "space-y-1",
            div { class: "text-xs font-semibold uppercase text-muted-foreground", "{label}" }
            if values.is_empty() {
                div { class: "text-sm text-muted-foreground", {t("policy.pin_no_public_summary")} }
            } else {
                div { class: "flex flex-wrap gap-1",
                    for value in values {
                        Badge {
                            variant: BadgeVariant::Secondary,
                            class: "font-mono text-xs".to_string(),
                            "{value}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PolicyGuardrailPanel(policy: AdminPolicy) -> Element {
    let required_scope = policy.guardrails.required_scope.clone();
    let evidence = policy.guardrails.approval_evidence.clone();
    let audit = policy.guardrails.audit_trail.clone();
    let obligations = policy.guardrails.obligations.len();
    let obligations_present = t("policy.obligations_present");

    rsx! {
        div {
            class: "space-y-3 rounded-md border border-border bg-muted/30 p-3",
            "data-testid": "policy-guardrail-panel",
            div { class: "flex flex-wrap items-center justify-between gap-2",
                h3 { class: "text-sm font-semibold", {t("policy.guardrails")} }
                PolicyGuardrailBadges { policy: policy.clone() }
            }
            if let Some(scope) = required_scope {
                div { class: "rounded-md border border-destructive/30 bg-destructive/5 p-2 text-sm",
                    div { class: "font-medium text-destructive", {t("policy.permission_required")} }
                    div { class: "mt-1 font-mono text-xs break-all", "{scope}" }
                }
            }
            div { class: "grid gap-3 md:grid-cols-2",
                div { class: "space-y-2",
                    div { class: "text-xs font-semibold uppercase text-muted-foreground", {t("policy.approval_evidence")} }
                    if evidence.is_empty() {
                        div { class: "text-sm text-muted-foreground", {t("policy.approval_evidence_empty")} }
                    } else {
                        for item in evidence {
                            PolicyEvidenceRow { item }
                        }
                    }
                }
                div { class: "space-y-2",
                    div { class: "text-xs font-semibold uppercase text-muted-foreground", {t("policy.audit_trail")} }
                    if audit.is_empty() {
                        div { class: "text-sm text-muted-foreground", {t("policy.audit_trail_empty")} }
                    } else {
                        for entry in audit {
                            PolicyAuditRow { entry }
                        }
                    }
                }
            }
            if obligations > 0 {
                div { class: "text-xs text-muted-foreground",
                    "{obligations} {obligations_present}"
                }
            }
        }
    }
}

#[component]
fn PolicyEvidenceRow(item: PolicyEvidenceItem) -> Element {
    let actor = item.actor.clone().unwrap_or_else(|| t("common.unknown"));
    let decision = item.decision.clone().unwrap_or_else(|| t("common.unknown"));
    let issued_at = item.issued_at.clone().unwrap_or_else(|| "-".to_owned());
    let approved_by_label = t("policy.approved_by");
    let decision_label = t("policy.decision");
    let issued_at_label = t("policy.issued_at");

    rsx! {
        div { class: "rounded-md border bg-background p-2 text-sm", "data-testid": "policy-approval-evidence",
            div { class: "flex flex-wrap items-center gap-2",
                Badge { variant: BadgeVariant::Secondary, "{item.kind}" }
                span { class: "font-mono text-xs break-all", "{item.reference}" }
            }
            div { class: "mt-1 grid gap-1 text-xs text-muted-foreground",
                span { "{approved_by_label}: {actor}" }
                span { "{decision_label}: {decision}" }
                span { "{issued_at_label}: {issued_at}" }
                if let Some(digest) = item.digest {
                    span { class: "font-mono break-all", "{digest}" }
                }
            }
        }
    }
}

#[component]
fn PolicyAuditRow(entry: PolicyAuditEntry) -> Element {
    let actor = entry.actor.clone().unwrap_or_else(|| t("common.unknown"));
    let outcome = entry.outcome.clone().unwrap_or_else(|| t("common.unknown"));
    let timestamp = entry.timestamp.clone().unwrap_or_else(|| "-".to_owned());
    let request_id = entry.request_id.clone().unwrap_or_else(|| "-".to_owned());
    let actor_label = t("policy.actor");
    let outcome_label = t("policy.outcome");
    let request_id_label = t("policy.request_id");

    rsx! {
        div { class: "rounded-md border bg-background p-2 text-sm", "data-testid": "policy-audit-entry",
            div { class: "flex flex-wrap items-center gap-2",
                Badge { variant: BadgeVariant::Outline, "{entry.action}" }
                span { class: "text-xs text-muted-foreground", "{timestamp}" }
            }
            div { class: "mt-1 grid gap-1 text-xs text-muted-foreground",
                span { "{actor_label}: {actor}" }
                span { "{outcome_label}: {outcome}" }
                span { class: "font-mono break-all", "{request_id_label}: {request_id}" }
            }
        }
    }
}

fn policy_mutation_error_message(error: &HttpError) -> String {
    if let Some(request_id) = error.request_id.as_deref() {
        format!("{} ({request_id})", error.message)
    } else {
        error.message.clone()
    }
}
