use dioxus::prelude::*;

use crate::api::agents;
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::info_row::InfoRow;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::router::Route;
use crate::utils::i18n::t;

#[component]
pub fn AgentShow(agent_id: String) -> Element {
    let mut agent_data = use_resource({
        let id = agent_id.clone();
        move || {
            let id = id.clone();
            async move { agents::get_agent(&id).await }
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
                                    InfoRow { label: t("agents.enabled"), value: if is_enabled { "Yes".to_string() } else { "No".to_string() } }
                                    InfoRow { label: t("agents.created_at"), value: created_at }
                                    InfoRow { label: t("agents.last_active_at"), value: last_active }
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
    }
}
