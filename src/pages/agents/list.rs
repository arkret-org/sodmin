use dioxus::prelude::*;

use crate::api::agents;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::utils::csv::{build_csv, export_to_csv};
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn AgentList() -> Element {
    let mut page = use_signal(|| 1u64);

    let page_val = *page.read();

    let mut data =
        use_resource(move || async move { agents::list_agents(page_val, PAGE_SIZE).await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("agents.title"),
                description: t("agents.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| {
                        if let Some(Ok(resp)) = data.read().as_ref() {
                            let rows: Vec<Vec<String>> = resp.data.iter().map(|a| vec![
                                a.id.clone(),
                                a.name.clone().unwrap_or_default(),
                                a.owner_id.clone(),
                                a.agent_type.clone().unwrap_or_default(),
                                a.status.clone().unwrap_or_default(),
                                if a.is_enabled { "true".into() } else { "false".into() },
                                a.last_active_at.clone().unwrap_or_default(),
                            ]).collect();
                            let csv = build_csv(
                                &["id", "name", "owner_id", "type", "status", "enabled", "last_active_at"],
                                &rows,
                            );
                            export_to_csv("agents.csv", &csv);
                            show_toast("Agents CSV downloaded", ToastVariant::Success);
                        }
                    },
                    {t("common.export_csv")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("agents.id")} }
                                    TableHead { {t("agents.name")} }
                                    TableHead { {t("agents.owner_id")} }
                                    TableHead { {t("agents.agent_type")} }
                                    TableHead { {t("agents.status")} }
                                    TableHead { {t("agents.enabled")} }
                                    TableHead { {t("agents.last_active_at")} }
                                    TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("agents.no_agents")}
                                        }
                                    }
                                } else {
                                    for agent in resp.data.iter() {
                                        {
                                            let id = agent.id.clone();
                                            let name = agent.name.clone().unwrap_or_else(|| "-".to_string());
                                            let owner_id = agent.owner_id.clone();
                                            let agent_type = agent.agent_type.clone().unwrap_or_else(|| "-".to_string());
                                            let status = agent.status.clone().unwrap_or_else(|| "-".to_string());
                                            let is_enabled = agent.is_enabled;
                                            let last_active = agent.last_active_at.clone().unwrap_or_else(|| "-".to_string());

                                            let id_for_toggle = id.clone();

                                            rsx! {
                                                TableRow {
                                                    TableCell {
                                                        Link {
                                                            to: Route::AgentShow { agent_id: id.clone() },
                                                            class: "font-medium text-primary hover:underline",
                                                            "{id}"
                                                        }
                                                    }
                                                    TableCell { "{name}" }
                                                    TableCell { class: "max-w-[200px] truncate".to_string(), "{owner_id}" }
                                                    TableCell { "{agent_type}" }
                                                    TableCell {
                                                        Badge { variant: BadgeVariant::Secondary, "{status}" }
                                                    }
                                                    TableCell {
                                                        if is_enabled {
                                                            Badge { variant: BadgeVariant::Success, {t("common.enabled")} }
                                                        } else {
                                                            Badge { variant: BadgeVariant::Default, {t("common.disabled")} }
                                                        }
                                                    }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{last_active}" }
                                                    TableCell { class: "text-right".to_string(),
                                                        div { class: "flex items-center justify-end gap-1",
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                size: ButtonSize::Sm,
                                                                disabled: is_enabled,
                                                                onclick: {
                                                                    let id = id_for_toggle.clone();
                                                                    move |_| {
                                                                        let id = id.clone();
                                                                        spawn(async move {
                                                                            match agents::enable_agent(&id).await {
                                                                                Ok(_) => {
                                                                                    show_toast("Agent enabled", ToastVariant::Success);
                                                                                    data.restart();
                                                                                }
                                                                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                {t("common.enable")}
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                size: ButtonSize::Sm,
                                                                disabled: !is_enabled,
                                                                onclick: {
                                                                    let id = id_for_toggle.clone();
                                                                    move |_| {
                                                                        let id = id.clone();
                                                                        spawn(async move {
                                                                            match agents::disable_agent(&id).await {
                                                                                Ok(_) => {
                                                                                    show_toast("Agent disabled", ToastVariant::Success);
                                                                                    data.restart();
                                                                                }
                                                                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                {t("common.disable")}
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
                        errcode: e.body.as_ref().map(|b| b.errcode.clone()),
                        request_id: e.request_id.clone(),
                        retry_after_ms: e.retry_after_ms,
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
