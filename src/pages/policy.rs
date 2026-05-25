use dioxus::prelude::*;

use crate::api::generated::soland_admin::CreatePolicyRequest;
use crate::api::policy;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn PolicyList() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut show_dialog = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| None::<String>);
    let mut editing_id = use_signal(|| None::<String>);
    let mut name = use_signal(String::new);
    let mut policy_type = use_signal(String::new);
    let mut scope = use_signal(String::new);
    let mut is_enabled = use_signal(|| true);
    let mut priority = use_signal(|| 0i32);
    let mut dialog_loading = use_signal(|| false);

    let page_val = *page.read();

    let mut data =
        use_resource(move || async move { policy::list_policies(page_val, PAGE_SIZE).await });

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
                        policy_type.set(String::new());
                        scope.set(String::new());
                        is_enabled.set(true);
                        priority.set(0);
                        show_dialog.set(true);
                    },
                    {t("common.create")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("policy.id")} }
                                    TableHead { {t("policy.name")} }
                                    TableHead { {t("policy.policy_type")} }
                                    TableHead { {t("policy.scope")} }
                                    TableHead { {t("policy.enabled")} }
                                    TableHead { {t("policy.priority")} }
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
                                            let p_type = p.policy_type.clone().unwrap_or_else(|| "-".to_string());
                                            let p_scope = p.scope.clone().unwrap_or_else(|| "-".to_string());
                                            let p_enabled = p.is_enabled;
                                            let p_priority = p.priority;
                                            let updated = p.updated_at.clone().unwrap_or_else(|| "-".to_string());

                                            let id_for_edit = id.clone();
                                            let id_for_delete = id.clone();

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { "{p_name}" }
                                                    TableCell { "{p_type}" }
                                                    TableCell { class: "max-w-[200px] truncate".to_string(), "{p_scope}" }
                                                    TableCell {
                                                        if p_enabled {
                                                            Badge { variant: BadgeVariant::Success, {t("common.enabled")} }
                                                        } else {
                                                            Badge { variant: BadgeVariant::Default, {t("common.disabled")} }
                                                        }
                                                    }
                                                    TableCell { "{p_priority}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{updated}" }
                                                    TableCell { class: "text-right".to_string(),
                                                        div { class: "flex items-center justify-end gap-1",
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                onclick: {
                                                                    let id = id_for_edit.clone();
                                                                    let n = p_name.clone();
                                                                    let t = p_type.clone();
                                                                    let s = p_scope.clone();
                                                                    let e = p_enabled;
                                                                    let pr = p_priority;
                                                                    move |_| {
                                                                        editing_id.set(Some(id.clone()));
                                                                        name.set(n.clone());
                                                                        policy_type.set(t.clone());
                                                                        scope.set(s.clone());
                                                                        is_enabled.set(e);
                                                                        priority.set(pr);
                                                                        show_dialog.set(true);
                                                                    }
                                                                },
                                                                {t("common.edit")}
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
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

        if *show_dialog.read() {
            div { class: "fixed inset-0 z-50 flex items-center justify-center",
                    div { class: "fixed inset-0 bg-black/80", onclick: move |_| show_dialog.set(false) }
                    div { class: "relative z-50 w-full max-w-md rounded-lg border glass-panel p-6 shadow-lg space-y-4",
                        h2 { class: "text-lg font-semibold",
                            if editing_id.read().is_some() {
                                {t("policy.edit")}
                            } else {
                                {t("policy.create")}
                            }
                        }
                        div { class: "space-y-3",
                            div { class: "space-y-1",
                                Label { r#for: "pol-name".to_string(), {t("policy.name")} }
                                Input {
                                    value: name.read().clone(),
                                    oninput: move |evt: FormEvent| name.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "pol-type".to_string(), {t("policy.policy_type")} }
                                Input {
                                    value: policy_type.read().clone(),
                                    oninput: move |evt: FormEvent| policy_type.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "pol-scope".to_string(), {t("policy.scope")} }
                                Input {
                                    value: scope.read().clone(),
                                    oninput: move |evt: FormEvent| scope.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "pol-priority".to_string(), {t("policy.priority")} }
                                Input {
                                    r#type: "number".to_string(),
                                    value: priority.read().to_string(),
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
                                    onchange: move |evt: Event<FormData>| is_enabled.set(evt.checked()),
                                }
                                Label { {t("policy.enabled")} }
                            }
                        }
                        div { class: "flex justify-end gap-2",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| show_dialog.set(false),
                                {t("common.cancel")}
                            }
                            Button {
                                variant: ButtonVariant::Default,
                                disabled: *dialog_loading.read(),
                                onclick: move |_| {
                                    dialog_loading.set(true);
                                    let req = CreatePolicyRequest {
                                        name: name.read().clone(),
                                        policy_type: if policy_type.read().is_empty() { None } else { Some(policy_type.read().clone()) },
                                        scope: if scope.read().is_empty() { None } else { Some(scope.read().clone()) },
                                        is_enabled: *is_enabled.read(),
                                        priority: *priority.read(),
                                        ..Default::default()
                                    };
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
                                                show_dialog.set(false);
                                                data.restart();
                                            }
                                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                        }
                                        dialog_loading.set(false);
                                    });
                                },
                                {t("common.save")}
                            }
                        }
                    }
                }
            }

        ConfirmDialog {
            open: show_delete_dialog.read().is_some(),
            title: t("common.delete"),
            description: "Are you sure you want to delete this policy?".to_string(),
            confirm_text: t("common.delete"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_delete_dialog.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match policy::delete_policy(&id).await {
                            Ok(_) => {
                                show_toast("Policy deleted", ToastVariant::Success);
                                data.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                    });
                }
                show_delete_dialog.set(None);
            },
            on_cancel: move |_| show_delete_dialog.set(None),
        }
    }
}
