use dioxus::prelude::*;

use crate::api::agents;
use crate::components::dangerous_action_dialog::DangerousActionDialog;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::utils::fmt::csv::{build_csv, export_to_csv};
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

/// R3 — Pick a colour for the agent FSM badge. The cokret-spec v3
/// canonical states are `active | paused | deactivated`; legacy
/// `enabled / disabled` rows fall back to the existing colour.
fn status_variant(status: &str) -> BadgeVariant {
    match status {
        "active" | "enabled" => BadgeVariant::Success,
        "paused" => BadgeVariant::Outline,
        "deactivated" | "revoked" => BadgeVariant::Destructive,
        _ => BadgeVariant::Secondary,
    }
}

#[component]
pub fn AgentList() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut deactivate_target = use_signal::<Option<String>>(|| None);

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
                            show_toast(&t("agents.toast_csv_downloaded"), ToastVariant::Success);
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
                                            let status = agent.status.clone().unwrap_or_else(|| {
                                                if agent.is_enabled { "active".to_string() } else { "deactivated".to_string() }
                                            });
                                            let last_active = agent.last_active_at.clone().unwrap_or_else(|| "-".to_string());

                                            let is_deactivated = status == "deactivated" || status == "revoked";
                                            let is_paused = status == "paused";
                                            let id_pause = id.clone();
                                            let id_resume = id.clone();
                                            let id_rotate = id.clone();
                                            let id_grants = id.clone();
                                            let id_deactivate = id.clone();
                                            let variant = status_variant(&status);

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
                                                        Badge { variant, "{status}" }
                                                    }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{last_active}" }
                                                    TableCell { class: "text-right".to_string(),
                                                        div { class: "flex items-center justify-end gap-1 flex-wrap",
                                                            // Pause — disabled when paused / deactivated.
                                                            Button {
                                                                variant: ButtonVariant::Outline,
                                                                size: ButtonSize::Sm,
                                                                disabled: is_paused || is_deactivated,
                                                                onclick: {
                                                                    let id = id_pause.clone();
                                                                    move |_| {
                                                                        let id = id.clone();
                                                                        spawn(async move {
                                                                            match agents::pause_personal_agent(&id).await {
                                                                                Ok(_) => {
                                                                                    show_toast(&t("agents.toast_paused"), ToastVariant::Success);
                                                                                    data.restart();
                                                                                }
                                                                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                {t("agents.pause")}
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Outline,
                                                                size: ButtonSize::Sm,
                                                                disabled: !is_paused,
                                                                onclick: {
                                                                    let id = id_resume.clone();
                                                                    move |_| {
                                                                        let id = id.clone();
                                                                        spawn(async move {
                                                                            match agents::resume_personal_agent(&id).await {
                                                                                Ok(_) => {
                                                                                    show_toast(&t("agents.toast_resumed"), ToastVariant::Success);
                                                                                    data.restart();
                                                                                }
                                                                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                {t("agents.resume")}
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Outline,
                                                                size: ButtonSize::Sm,
                                                                disabled: is_deactivated,
                                                                onclick: {
                                                                    let id = id_rotate.clone();
                                                                    move |_| {
                                                                        let id = id.clone();
                                                                        spawn(async move {
                                                                            match agents::rotate_personal_agent_key(&id).await {
                                                                                Ok(_) => {
                                                                                    show_toast(&t("agents.toast_key_rotated"), ToastVariant::Success);
                                                                                    data.restart();
                                                                                }
                                                                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                {t("agents.rotate_key")}
                                                            }
                                                            Link {
                                                                to: Route::AgentShow { agent_id: id_grants.clone() },
                                                                class: "inline-flex items-center px-2 py-1 text-xs rounded border hover:bg-accent",
                                                                {t("agents.grants")}
                                                            }
                                                            // Destructive — gated by a typed-keyword
                                                            // confirmation dialog. Hits the canonical
                                                            // `/agents/{id}/deactivate` HTTP path; the
                                                            // legacy `/revoke` form is no longer used.
                                                            Button {
                                                                variant: ButtonVariant::Destructive,
                                                                size: ButtonSize::Sm,
                                                                disabled: is_deactivated,
                                                                onclick: {
                                                                    let id = id_deactivate.clone();
                                                                    move |_| deactivate_target.set(Some(id.clone()))
                                                                },
                                                                {t("agents.deactivate")}
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
                        total: resp.total_or_len(),
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

            DangerousActionDialog {
                open: deactivate_target.read().is_some(),
                title: t("agents.deactivate_title"),
                description: t("agents.deactivate_body"),
                confirmation_phrase: "DEACTIVATE".to_string(),
                confirm_text: t("agents.deactivate"),
                on_cancel: move |_| deactivate_target.set(None),
                on_confirm: move |_| {
                    let target = deactivate_target.read().clone();
                    if let Some(id) = target {
                        deactivate_target.set(None);
                        spawn(async move {
                            // R3 — canonical HTTP path is
                            // /_cokret/self/agents/{id}/deactivate. The
                            // legacy /revoke shape is gone.
                            match agents::deactivate_personal_agent(&id).await {
                                Ok(_) => {
                                    show_toast(&t("agents.toast_deactivated"), ToastVariant::Success);
                                    data.restart();
                                }
                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                            }
                        });
                    }
                },
            }
        }
    }
}
