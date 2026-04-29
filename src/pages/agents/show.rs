use dioxus::prelude::*;

use crate::api::agents;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::utils::i18n::t;

const MEMORY_PAGE_SIZE: u64 = 25;

#[component]
pub fn AgentShow(agent_id: String) -> Element {
    let mut memory_page = use_signal(|| 1u64);
    let mut show_delete_memory = use_signal(|| None::<(String, String)>);

    let mut agent_data = use_resource({
        let id = agent_id.clone();
        move || {
            let id = id.clone();
            async move { agents::get_agent(&id).await }
        }
    });

    let memory_page_val = *memory_page.read();

    let mut memory_data = use_resource({
        let id = agent_id.clone();
        move || {
            let id = id.clone();
            async move { agents::list_agent_memory(&id, memory_page_val, MEMORY_PAGE_SIZE).await }
        }
    });

    rsx! {
        div { class: "space-y-6",
            Breadcrumbs {
                items: vec![
                    BreadcrumbItem { label: t("agents.title"), route: Some(Route::AgentList {}) },
                    BreadcrumbItem { label: agent_id.clone(), route: None },
                ],
            }

            match &*agent_data.read() {
                Some(Ok(agent)) => {
                    let name = agent.name.clone().unwrap_or_else(|| "-".to_string());
                    let owner_id = agent.owner_id.clone();
                    let agent_type = agent.agent_type.clone().unwrap_or_else(|| "-".to_string());
                    let status = agent.status.clone().unwrap_or_else(|| "-".to_string());
                    let memory_count = agent.memory_count;
                    let is_enabled = agent.is_enabled;
                    let created_at = agent.created_at.clone().unwrap_or_else(|| "-".to_string());
                    let last_active = agent.last_active_at.clone().unwrap_or_else(|| "-".to_string());

                    rsx! {
                        PageHeader {
                            title: format!("Agent: {name}"),
                            description: agent.id.clone(),
                        }

                        Card {
                            CardHeader { CardTitle { {t("agents.agent_info")} } }
                            CardContent {
                                div { class: "space-y-4",
                                    InfoRow { label: t("agents.id"), value: agent.id.clone() }
                                    InfoRow { label: t("agents.name"), value: name }
                                    InfoRow { label: t("agents.owner_id"), value: owner_id }
                                    InfoRow { label: t("agents.agent_type"), value: agent_type }
                                    InfoRow { label: t("agents.status"), value: status }
                                    InfoRow { label: t("agents.memory_count"), value: memory_count.to_string() }
                                    InfoRow { label: t("agents.enabled"), value: if is_enabled { "Yes".to_string() } else { "No".to_string() } }
                                    InfoRow { label: t("agents.created_at"), value: created_at }
                                    InfoRow { label: t("agents.last_active_at"), value: last_active }
                                }
                            }
                        }

                        Card {
                            CardHeader { CardTitle { {t("agents.memory")} } }
                            CardContent {
                                match &*memory_data.read() {
                                    Some(Ok(mem)) => rsx! {
                                        div { class: "rounded-md border",
                                            Table {
                                                TableHeader {
                                                    TableRow {
                                                        TableHead { {t("agents.memory_id")} }
                                                        TableHead { {t("agents.memory_type")} }
                                                        TableHead { {t("agents.confidence")} }
                                                        TableHead { {t("agents.status")} }
                                                        TableHead { {t("agents.content_summary")} }
                                                        TableHead { {t("agents.created_at")} }
                                                        TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                                    }
                                                }
                                                TableBody {
                                                    if mem.data.is_empty() {
                                                        TableRow {
                                                            TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                                {t("agents.no_memory")}
                                                            }
                                                        }
                                                    } else {
                                                        for entry in mem.data.iter() {
                                                            {
                                                                let mem_id = entry.id.clone();
                                                                let mem_type = entry.memory_type.clone().unwrap_or_else(|| "-".to_string());
                                                                let confidence = entry.confidence.map(|c| format!("{:.2}", c)).unwrap_or_else(|| "-".to_string());
                                                                let mem_status = entry.status.clone().unwrap_or_else(|| "-".to_string());
                                                                let summary = entry.content_summary.clone().unwrap_or_else(|| "-".to_string());
                                                                let created = entry.created_at.clone().unwrap_or_else(|| "-".to_string());

                                                                let agent_id_del = agent.id.clone();
                                                                let mem_id_del = mem_id.clone();

                                                                rsx! {
                                                                    TableRow {
                                                                        TableCell { class: "font-medium".to_string(), "{mem_id}" }
                                                                        TableCell { "{mem_type}" }
                                                                        TableCell { "{confidence}" }
                                                                        TableCell { "{mem_status}" }
                                                                        TableCell { class: "max-w-[300px] truncate".to_string(), "{summary}" }
                                                                        TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                                        TableCell { class: "text-right".to_string(),
                                                                            Button {
                                                                                variant: ButtonVariant::Ghost,
                                                                                size: ButtonSize::Sm,
                                                                                onclick: {
                                                                                    let aid = agent_id_del.clone();
                                                                                    let mid = mem_id_del.clone();
                                                                                    move |_| show_delete_memory.set(Some((aid.clone(), mid.clone())))
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
                                        Pagination {
                                            page: memory_page_val,
                                            total: mem.total,
                                            per_page: MEMORY_PAGE_SIZE,
                                            on_page_change: move |p| memory_page.set(p),
                                        }
                                    },
                                    Some(Err(e)) => rsx! {
                                        ErrorBanner { message: e.message.clone(), on_retry: move |_| memory_data.restart() }
                                    },
                                    None => rsx! { PageSkeleton {} },
                                }
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        on_retry: move |_| agent_data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }

        ConfirmDialog {
            open: show_delete_memory.read().is_some(),
            title: t("common.delete"),
            description: "Are you sure you want to delete this memory entry?".to_string(),
            confirm_text: t("common.delete"),
            destructive: true,
            on_confirm: move |_| {
                if let Some((aid, mid)) = show_delete_memory.read().clone() {
                    spawn(async move {
                        match agents::delete_agent_memory(&aid, &mid).await {
                            Ok(_) => {
                                show_toast("Memory deleted", ToastVariant::Success);
                                memory_data.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                    });
                }
                show_delete_memory.set(None);
            },
            on_cancel: move |_| show_delete_memory.set(None),
        }
    }
}

#[component]
fn InfoRow(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between py-2",
            span { class: "text-sm font-medium text-muted-foreground", "{label}" }
            span { class: "text-sm max-w-[60%] text-right break-all", "{value}" }
        }
    }
}
