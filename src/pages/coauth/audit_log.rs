use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[derive(Debug, Clone, Default, PartialEq)]
struct DraftFilter {
    operation: String,
    actor_user_id: String,
    target_type: String,
    target_id: String,
    since: String,
    until: String,
}

impl DraftFilter {
    fn to_query(&self) -> coauth::AuditFeedFilter {
        coauth::AuditFeedFilter {
            operation: trim_to_option(&self.operation),
            actor_user_id: trim_to_option(&self.actor_user_id),
            target_type: trim_to_option(&self.target_type),
            target_id: trim_to_option(&self.target_id),
            since: trim_to_option(&self.since),
            until: trim_to_option(&self.until),
        }
    }

    fn is_empty(&self) -> bool {
        self.operation.trim().is_empty()
            && self.actor_user_id.trim().is_empty()
            && self.target_type.trim().is_empty()
            && self.target_id.trim().is_empty()
            && self.since.trim().is_empty()
            && self.until.trim().is_empty()
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
pub fn AuditLogPage() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut draft = use_signal(DraftFilter::default);
    let mut applied = use_signal(DraftFilter::default);

    let mut data = use_resource(move || {
        let page_val = *page.read();
        let filter = applied.read().clone();
        async move { coauth::list_audit_feed(page_val, PAGE_SIZE, filter.to_query()).await }
    });
    let page_val = *page.read();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth.audit_log.title"),
                description: t("coauth.audit_log.subtitle"),
            }

            div { class: "rounded-md border bg-card p-4",
                div { class: "grid gap-3 md:grid-cols-3",
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("coauth.audit_log.filter_operation")} }
                        Input {
                            placeholder: t("coauth.audit_log.filter_operation_placeholder"),
                            value: draft.read().operation.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().operation = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("coauth.audit_log.filter_actor")} }
                        Input {
                            placeholder: t("coauth.audit_log.filter_actor_placeholder"),
                            value: draft.read().actor_user_id.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().actor_user_id = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("coauth.audit_log.filter_target_type")} }
                        Input {
                            placeholder: t("coauth.audit_log.filter_target_type_placeholder"),
                            value: draft.read().target_type.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().target_type = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("coauth.audit_log.filter_target_id")} }
                        Input {
                            placeholder: t("coauth.audit_log.filter_target_id_placeholder"),
                            value: draft.read().target_id.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().target_id = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("coauth.audit_log.filter_since")} }
                        Input {
                            r#type: "datetime-local".to_string(),
                            value: draft.read().since.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().since = evt.value();
                            },
                        }
                    }
                    div { class: "space-y-1",
                        Label { class: "text-xs text-muted-foreground".to_string(), {t("coauth.audit_log.filter_until")} }
                        Input {
                            r#type: "datetime-local".to_string(),
                            value: draft.read().until.clone(),
                            oninput: move |evt: FormEvent| {
                                draft.write().until = evt.value();
                            },
                        }
                    }
                }
                div { class: "mt-4 flex flex-wrap items-center gap-2",
                    Button {
                        onclick: move |_| {
                            page.set(1);
                            applied.set(draft.read().clone());
                        },
                        {t("coauth.audit_log.filter_apply")}
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        disabled: draft.read().is_empty() && applied.read().is_empty(),
                        onclick: move |_| {
                            draft.set(DraftFilter::default());
                            applied.set(DraftFilter::default());
                            page.set(1);
                        },
                        {t("coauth.audit_log.filter_reset")}
                    }
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("coauth.audit_log.id")} }
                                    TableHead { {t("coauth.audit_log.operation")} }
                                    TableHead { {t("coauth.audit_log.actor")} }
                                    TableHead { {t("coauth.audit_log.target_type")} }
                                    TableHead { {t("coauth.audit_log.target_id")} }
                                    TableHead { {t("coauth.audit_log.col_detail")} }
                                    TableHead { {t("coauth.audit_log.timestamp")} }
                                    TableHead { {t("coauth.audit_log.source_ip")} }
                                    TableHead { {t("coauth.audit_log.signature_status")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("coauth.audit_log.no_entries")}
                                        }
                                    }
                                } else {
                                    for entry in resp.data.iter() {
                                        {
                                            let id = entry.id.clone();
                                            let op = entry.operation.clone();
                                            let actor = entry.actor_user_id.clone().unwrap_or_else(|| "-".to_string());
                                            let target_type = entry.target_type.clone().unwrap_or_else(|| "-".to_string());
                                            let target_id = entry.target_id.clone().unwrap_or_else(|| "-".to_string());
                                            let details = audit_details_text(&entry.details);
                                            let ts = entry.timestamp.clone().unwrap_or_else(|| "-".to_string());
                                            let ip = entry.source_ip.clone().unwrap_or_else(|| "-".to_string());
                                            let signature_status = format!("{:?}", entry.signature_status);

                                            rsx! {
                                                TableRow {
                                                    key: "{id}",
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { "{op}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{actor}" }
                                                    TableCell { "{target_type}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{target_id}" }
                                                    TableCell { class: "max-w-[280px] whitespace-pre-wrap break-all text-xs text-muted-foreground".to_string(), "{details}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{ts}" }
                                                    TableCell { "{ip}" }
                                                    TableCell { "{signature_status}" }
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

fn audit_details_text(details: &Option<serde_json::Value>) -> String {
    match details {
        Some(value) if !value.is_null() => {
            serde_json::to_string(value).unwrap_or_else(|_| "-".to_string())
        }
        _ => "-".to_string(),
    }
}
