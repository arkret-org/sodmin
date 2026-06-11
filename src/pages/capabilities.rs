use dioxus::prelude::*;

use crate::api::capabilities;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::modal::{DialogActions, Modal};
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::{
    CAPABILITY_GRANT_SCHEMA, GrantCapabilityRequest, UpdateCapabilityRequest,
    capability_resources_from_input,
};
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn CapabilityList() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut show_grant_dialog = use_signal(|| false);
    let mut show_revoke_dialog = use_signal(|| None::<String>);
    let mut show_edit_dialog = use_signal(|| None::<String>);
    let mut grantee_id = use_signal(String::new);
    let mut capability_name = use_signal(String::new);
    let mut scope = use_signal(String::new);
    let mut expires_at = use_signal(String::new);
    let mut grant_loading = use_signal(|| false);

    // P3A.4 — CKP-0007 Circle capability picker. Surfaces the six
    // ck.circle.* actions as a quick-select and exposes the
    // `allowed_circle_ids` constraint editor (CSV of ck:circle:...
    // ids). For actions whose `required_constraints` include
    // `allowed_circle_ids` (manage / member.manage / member.add.others)
    // the CSV is non-optional — soland's reducer rejects unconstrained
    // grants for those actions with `validation`.
    let mut circle_allowed_ids = use_signal(String::new);

    // T6.2 §5 — constraint editor signals.
    let mut edit_expires_at = use_signal(String::new);
    let mut edit_allowed_write_fields = use_signal(String::new);
    let mut edit_facets_allow = use_signal(String::new);
    let mut edit_approval_required = use_signal(|| false);
    let mut edit_loading = use_signal(|| false);

    let page_val = *page.read();

    let mut data =
        use_resource(
            move || async move { capabilities::list_capabilities(page_val, PAGE_SIZE).await },
        );

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("capabilities.title"),
                description: t("capabilities.subtitle"),
                Button {
                    variant: ButtonVariant::Default,
                    onclick: move |_| show_grant_dialog.set(true),
                    {t("capabilities.grant")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("capabilities.id")} }
                                    TableHead { {t("capabilities.grantor_id")} }
                                    TableHead { {t("capabilities.grantee_id")} }
                                    TableHead { {t("capabilities.capability")} }
                                    TableHead { {t("capabilities.scope")} }
                                    TableHead { {t("capabilities.granted_at")} }
                                    TableHead { {t("capabilities.expires_at")} }
                                    TableHead { {t("capabilities.revoked")} }
                                    TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("capabilities.no_capabilities")}
                                        }
                                    }
                                } else {
                                    for cap in resp.data.iter() {
                                        {
                                            let id = cap.id.clone();
                                            let issuer = cap.issuer.clone();
                                            let subject = cap.subject.clone();
                                            let actions = cap.actions_display();
                                            let resources = cap.resources_display();
                                            let issued_at = cap.issued_at.clone().unwrap_or_else(|| "-".to_string());
                                            let expires = cap.expires_at.clone().unwrap_or_else(|| "-".to_string());
                                            let is_revoked = cap.is_revoked();

                                            let id_for_revoke = id.clone();

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{issuer}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{subject}" }
                                                    TableCell { "{actions}" }
                                                    TableCell { class: "max-w-[200px] truncate".to_string(), "{resources}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{issued_at}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{expires}" }
                                                    TableCell {
                                                        if is_revoked {
                                                            Badge { variant: BadgeVariant::Destructive, {t("capabilities.revoked")} }
                                                        } else {
                                                            Badge { variant: BadgeVariant::Success, {t("capabilities.active")} }
                                                        }
                                                    }
                                                    TableCell { class: "text-right space-x-1".to_string(),
                                                        Button {
                                                            variant: ButtonVariant::Ghost,
                                                            size: ButtonSize::Sm,
                                                            disabled: is_revoked,
                                                            onclick: {
                                                                let id = id_for_revoke.clone();
                                                                let constraints = cap.constraints.clone();
                                                                let exp = cap.expires_at.clone().unwrap_or_default();
                                                                move |_| {
                                                                    // Hydrate edit signals from the
                                                                    // existing grant. `constraints`
                                                                    // is a free-form `serde_json::Value`,
                                                                    // so we pluck the common keys we
                                                                    // surface in the form.
                                                                    edit_expires_at.set(exp.clone());
                                                                    let mut fields = Vec::<String>::new();
                                                                    let mut facets = Vec::<String>::new();
                                                                    let mut approval = false;
                                                                    for constraint in constraints.iter() {
                                                                        let Some(map) = constraint.as_object() else {
                                                                            continue;
                                                                        };
                                                                        if let Some(serde_json::Value::Array(arr)) = map.get("allowed_write_fields") {
                                                                            fields = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                                                                        }
                                                                        if let Some(serde_json::Value::Array(arr)) = map.get("facets_allow") {
                                                                            facets = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                                                                        }
                                                                        if let Some(serde_json::Value::Bool(b)) = map.get("approval_required") {
                                                                            approval = *b;
                                                                        }
                                                                    }
                                                                    edit_allowed_write_fields.set(fields.join(", "));
                                                                    edit_facets_allow.set(facets.join(", "));
                                                                    edit_approval_required.set(approval);
                                                                    show_edit_dialog.set(Some(id.clone()));
                                                                }
                                                            },
                                                            {t("common.edit")}
                                                        }
                                                        Button {
                                                            variant: ButtonVariant::Ghost,
                                                            size: ButtonSize::Sm,
                                                            disabled: is_revoked,
                                                            onclick: {
                                                                let id = id_for_revoke.clone();
                                                                move |_| show_revoke_dialog.set(Some(id.clone()))
                                                            },
                                                            {t("capabilities.revoke")}
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

        Modal {
            open: *show_grant_dialog.read(),
            title: t("capabilities.grant"),
            on_close: move |_| show_grant_dialog.set(false),
            div { class: "space-y-3",
                div { class: "space-y-1",
                    Label { r#for: "cap-grantee".to_string(), {t("capabilities.grantee_id")} }
                    Input {
                        value: grantee_id.read().clone(),
                        oninput: move |evt: FormEvent| grantee_id.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "cap-name".to_string(), {t("capabilities.capability")} }
                    Input {
                        value: capability_name.read().clone(),
                        oninput: move |evt: FormEvent| capability_name.set(evt.value()),
                    }
                }
                // P3A.4 — Quick-select for the 6 CKP-0007
                // ck.circle.* actions. Selecting one
                // populates the capability_name field.
                div { class: "space-y-1",
                    Label { r#for: "cap-circle-quick".to_string(),
                        {t("capability.ck_circle_section")}
                    }
                    select {
                        id: "cap-circle-quick",
                        name: "ck_circle_action",
                        class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                        onchange: move |e| {
                            if !e.value().is_empty() {
                                capability_name.set(e.value());
                            }
                        },
                        option { value: "", "— {t(\"capability.ck_circle_section\")} —" }
                        option { value: "ck.circle.create", "ck.circle.create" }
                        option { value: "ck.circle.manage", "ck.circle.manage" }
                        option { value: "ck.circle.member.add", "ck.circle.member.add" }
                        option { value: "ck.circle.member.manage", "ck.circle.member.manage" }
                        option {
                            value: "ck.circle.member.add.others",
                            "ck.circle.member.add.others"
                        }
                        option { value: "ck.circle.audit", "ck.circle.audit" }
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "cap-allowed-circle-ids".to_string(),
                        {t("capability.allowed_circle_ids")}
                    }
                    Input {
                        value: circle_allowed_ids.read().clone(),
                        placeholder: "ck:circle:...,ck:circle:...".to_string(),
                        oninput: move |evt: FormEvent| circle_allowed_ids.set(evt.value()),
                    }
                    p { class: "text-xs text-muted-foreground",
                        {t("capability.allowed_circle_ids_hint")}
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "cap-scope".to_string(), {t("capabilities.scope")} }
                    Input {
                        value: scope.read().clone(),
                        oninput: move |evt: FormEvent| scope.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "cap-expires".to_string(), {t("capabilities.expires_at")} }
                    Input {
                        r#type: "datetime-local".to_string(),
                        value: expires_at.read().clone(),
                        oninput: move |evt: FormEvent| expires_at.set(evt.value()),
                    }
                }
            }
            DialogActions {
                confirm_text: t("capabilities.grant"),
                cancel_text: t("common.cancel"),
                confirm_loading: *grant_loading.read(),
                on_cancel: move |_| show_grant_dialog.set(false),
                on_confirm: move |_| {
                    grant_loading.set(true);
                                    // P3A.4 — pack allowed_circle_ids
                                    // into GrantConstraint when the
                                    // operator filled the CSV. soland's
                                    // authz layer reads
                                    // constraints.allowed_circle_ids
                                    // verbatim.
                                    let circle_ids: Vec<String> = circle_allowed_ids
                                        .read()
                                        .split(',')
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    let constraints = if circle_ids.is_empty() {
                                        Vec::new()
                                    } else {
                                        vec![serde_json::json!({
                                            "allowed_circle_ids": circle_ids,
                                        })]
                                    };
                                    let action = capability_name.read().trim().to_string();
                                    let req = GrantCapabilityRequest {
                                        schema: CAPABILITY_GRANT_SCHEMA.to_string(),
                                        subject: grantee_id.read().trim().to_string(),
                                        actions: if action.is_empty() { Vec::new() } else { vec![action] },
                                        resources: capability_resources_from_input(&scope.read()),
                                        constraints,
                                        parent_grant_id: None,
                                        not_before: None,
                                        expires_at: if expires_at.read().is_empty() { None } else { Some(expires_at.read().clone()) },
                                    };
                                    spawn(async move {
                                        match capabilities::grant_capability(&req).await {
                                            Ok(_) => {
                                                show_toast("Capability granted", ToastVariant::Success);
                                                show_grant_dialog.set(false);
                                                data.restart();
                                            }
                                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                        }
                        grant_loading.set(false);
                    });
                },
            }
        }

        Modal {
            open: show_edit_dialog.read().is_some(),
            title: t("capabilities.edit_title"),
            on_close: move |_| show_edit_dialog.set(None),
            div { class: "space-y-3",
                        div { class: "space-y-1",
                            Label { r#for: "cap-edit-expires".to_string(), {t("capabilities.expires_at")} }
                            Input {
                                r#type: "datetime-local".to_string(),
                                value: edit_expires_at.read().clone(),
                                oninput: move |evt: FormEvent| edit_expires_at.set(evt.value()),
                            }
                        }
                        div { class: "space-y-1",
                            Label { r#for: "cap-edit-fields".to_string(), {t("capabilities.allowed_write_fields")} }
                            Input {
                                value: edit_allowed_write_fields.read().clone(),
                                oninput: move |evt: FormEvent| edit_allowed_write_fields.set(evt.value()),
                            }
                            p { class: "text-xs text-muted-foreground", {t("capabilities.csv_hint")} }
                        }
                        div { class: "space-y-1",
                            Label { r#for: "cap-edit-facets".to_string(), {t("capabilities.facets_allow")} }
                            Input {
                                value: edit_facets_allow.read().clone(),
                                oninput: move |evt: FormEvent| edit_facets_allow.set(evt.value()),
                            }
                            p { class: "text-xs text-muted-foreground", {t("capabilities.csv_hint")} }
                        }
                        label {
                            class: "flex items-center gap-2 text-sm",
                            input {
                                r#type: "checkbox",
                                checked: *edit_approval_required.read(),
                                oninput: move |evt: FormEvent| {
                                    edit_approval_required.set(evt.value() == "true");
                                },
                            }
                            span { {t("capabilities.approval_required")} }
                        }
                    }
                    DialogActions {
                        confirm_text: t("common.save"),
                        cancel_text: t("common.cancel"),
                        confirm_loading: *edit_loading.read(),
                        on_cancel: move |_| show_edit_dialog.set(None),
                        on_confirm: move |_| {
                            if let Some(id) = show_edit_dialog.read().clone() {
                                let exp_raw = edit_expires_at.read().trim().to_string();
                                let fields: Vec<String> = edit_allowed_write_fields
                                    .read()
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                                let facets: Vec<String> = edit_facets_allow
                                    .read()
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                                let approval = *edit_approval_required.read();
                                let req = UpdateCapabilityRequest {
                                    expires_at: if exp_raw.is_empty() { None } else { Some(exp_raw) },
                                    allowed_write_fields: Some(fields),
                                    facets_allow: Some(facets),
                                    approval_required: Some(approval),
                                };
                                edit_loading.set(true);
                                spawn(async move {
                                    match capabilities::update_capability(&id, &req).await {
                                        Ok(_) => {
                                            show_toast(&t("capabilities.edit_ok"), ToastVariant::Success);
                                            show_edit_dialog.set(None);
                                            data.restart();
                                        }
                                        Err(e) => show_toast(&format!("{}: {}", t("capabilities.edit_fail"), e.message), ToastVariant::Error),
                                    }
                                    edit_loading.set(false);
                                });
                            }
                        },
                    }
                }

        ConfirmDialog {
            open: show_revoke_dialog.read().is_some(),
            title: t("capabilities.revoke"),
            description: "Are you sure you want to revoke this capability?".to_string(),
            confirm_text: t("capabilities.revoke"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_revoke_dialog.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match capabilities::revoke_capability(&id).await {
                            Ok(_) => {
                                show_toast("Capability revoked", ToastVariant::Success);
                                data.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                    });
                }
                show_revoke_dialog.set(None);
            },
            on_cancel: move |_| show_revoke_dialog.set(None),
        }
    }
}
