use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::claims_panel::ClaimsPanel;
use crate::components::did_binding_panel::DidBindingPanel;
use crate::components::risk_action_panel;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::pages::coauth::recovery_bridge_panel;
use crate::router::Route;

#[component]
pub fn AccountDetailPage(account_id: String) -> Element {
    let account_id_for_resource = account_id.clone();
    let mut data = use_resource(move || {
        let account_id_for_resource = account_id_for_resource.clone();
        async move { coauth::get_account_detail(&account_id_for_resource).await }
    });
    // Mutation panels (DID binding add/remove, claim revoke) need to
    // restart this resource on success; clone the signal so each panel
    // gets its own owned handle.
    let mut data_for_dids = data;
    let mut data_for_claims = data;

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "coauth Account Detail".to_string(),
                description: "Account-first admin detail view backed by coauth account, DID-binding, and explicit risk-action state-machine endpoints. Claims and grant inventory remain partial until backend coverage expands.".to_string(),
                Link {
                    to: Route::CoauthAccountList {},
                    class: "inline-flex h-10 items-center rounded-md border px-4 text-sm font-medium transition-colors hover:bg-accent".to_string(),
                    "Back to accounts"
                }
            }

            match &*data.read() {
                Some(Ok(detail)) => {
                    let summary = &detail.account;
                    let display_name = summary.display_name.clone().unwrap_or_else(|| summary.id.clone());
                    let handle = summary.username.clone().unwrap_or_else(|| "-".to_string());
                    let email = summary.email.clone().unwrap_or_else(|| "-".to_string());
                    let created_at = summary.created_at.clone().unwrap_or_else(|| "-".to_string());
                    let updated_at = summary.updated_at.clone().unwrap_or_else(|| "-".to_string());
                    let primary_did = summary.primary_did.clone().unwrap_or_else(|| "-".to_string());
                    let lifecycle = if summary.is_deactivated {
                        "Deactivated"
                    } else if summary.is_locked {
                        "Locked"
                    } else {
                        "Active"
                    };
                    let integration_dependencies = if detail.integration_manifest.dependencies.is_empty() {
                        "none".to_string()
                    } else {
                        detail
                            .integration_manifest
                            .dependencies
                            .iter()
                            .map(|dependency| {
                                format!(
                                    "{}:{}@{}",
                                    dependency.service, dependency.purpose, dependency.discovery_path
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" | ")
                    };
                    let integration_surfaces = if detail.integration_manifest.surfaces.is_empty() {
                        "none".to_string()
                    } else {
                        detail
                            .integration_manifest
                            .surfaces
                            .iter()
                            .map(|surface| {
                                format!(
                                    "{} {} {} [{}]",
                                    surface.method, surface.path, surface.contract, surface.stability
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" | ")
                    };
                    let integration_todos = if detail.integration_manifest.todos.is_empty() {
                        "none".to_string()
                    } else {
                        detail.integration_manifest.todos.join(" ")
                    };

                    rsx! {
                        div { class: "rounded-lg border p-4 space-y-2",
                            div { class: "text-lg font-semibold", "{display_name}" }
                            div { class: "text-sm text-muted-foreground",
                                "Bridge status: "
                                span { class: "font-mono", "{summary.bridge_status}" }
                            }
                            div { class: "text-sm text-muted-foreground",
                                "This panel is now backed by coauth admin bridge discovery plus account, DID-binding, claims, session-grant, and risk-action endpoints. High-risk actions flow through a discovered persisted state-machine scaffold. TODO(contract): replace scaffold transitions with controlled mutation executors."
                            }
                        }

                        div { class: "grid gap-4 md:grid-cols-2",
                            {section_block(
                                "Account Summary",
                                rsx! {
                                    {detail_row("Account ID", &summary.id)}
                                    {detail_row("Handle", &handle)}
                                    {detail_row("Email", &email)}
                                    {detail_row("Primary DID", &primary_did)}
                                },
                            )}
                            {section_block(
                                "Lifecycle",
                                rsx! {
                                    {detail_row("Status", lifecycle)}
                                    {detail_row("Locked", bool_label(summary.is_locked))}
                                    {detail_row("Deactivated", bool_label(summary.is_deactivated))}
                                    {detail_row("Created At", &created_at)}
                                    {detail_row("Updated At", &updated_at)}
                                },
                            )}
                        }

                        DidBindingPanel {
                            account_id: account_id.clone(),
                            bindings: detail.managed_dids.clone(),
                            on_mutated: move |_| data_for_dids.restart(),
                        }

                        ClaimsPanel {
                            account_id: account_id.clone(),
                            claims: detail.claims.clone(),
                            on_mutated: move |_| data_for_claims.restart(),
                        }

                        {section_block(
                            "Session Grants",
                            if detail.session_grants.is_empty() {
                                rsx! {
                                    p { class: "text-sm text-muted-foreground",
                                        "Session-grant inventory is not exposed by the current coauth account admin contract yet."
                                    }
                                }
                            } else {
                                rsx! {
                                    ul { class: "space-y-2",
                                        for grant in detail.session_grants.iter() {
                                            {
                                                let subject = grant.subject.clone().unwrap_or_else(|| "-".to_string());
                                                let scope = grant.scope.clone().unwrap_or_else(|| "-".to_string());
                                                let state = grant.state.clone().unwrap_or_else(|| "-".to_string());
                                                let issued_at = grant.issued_at.clone().unwrap_or_else(|| "-".to_string());
                                                rsx! {
                                                    li { class: "rounded-md border p-3",
                                                        div { class: "font-mono text-sm", "{grant.grant_id}" }
                                                        div { class: "mt-2 grid gap-2 text-sm md:grid-cols-2",
                                                            {detail_row("Subject", &subject)}
                                                            {detail_row("Scope", &scope)}
                                                            {detail_row("State", &state)}
                                                            {detail_row("Issued At", &issued_at)}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                        )}

                        {risk_action_panel::risk_action_panel(
                            &account_id,
                            &detail.risk_action_current,
                            &detail.risk_action_history,
                            &detail.risk_action_hook,
                            &detail.admin_bridge,
                        )}

                        div { class: "rounded-lg border p-4 space-y-4",
                            h2 { class: "text-base font-semibold", "Service Integration Manifest" }
                            p { class: "text-sm text-muted-foreground",
                                "Contract: "
                                span { class: "font-mono", "{detail.integration_manifest.contract}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Version: "
                                span { class: "font-mono", "{detail.integration_manifest.version}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Service: "
                                span { class: "font-mono", "{detail.integration_manifest.service}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Kind: "
                                span { class: "font-mono", "{detail.integration_manifest.service_kind}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Describe Path: "
                                span { class: "font-mono", "{detail.integration_manifest.describe_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Dependencies: "
                                span { class: "font-mono", "{integration_dependencies}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Surfaces: "
                                span { class: "font-mono", "{integration_surfaces}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "{integration_todos}"
                            }
                            h2 { class: "text-base font-semibold", "Admin Bridge Contract" }
                            p { class: "text-sm text-muted-foreground",
                                "Contract: "
                                span { class: "font-mono", "{detail.admin_bridge.contract}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Version: "
                                span { class: "font-mono", "{detail.admin_bridge.version}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "API Base: "
                                span { class: "font-mono", "{detail.admin_bridge.api_base_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Accounts: "
                                span { class: "font-mono", "{detail.admin_bridge.accounts_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Account Detail Template: "
                                span { class: "font-mono", "{detail.admin_bridge.account_detail_path_template}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "DID Bindings Template: "
                                span { class: "font-mono", "{detail.admin_bridge.account_dids_path_template}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Claims Template: "
                                span { class: "font-mono", "{detail.admin_bridge.account_claims_path_template}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Session Grants Template: "
                                span { class: "font-mono", "{detail.admin_bridge.account_session_grants_path_template}" }
                            }
                            {recovery_bridge_panel::recovery_bridge_block(&detail.recovery_bridge)}
                        }
                    }
                }
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        errcode: e.body.as_ref().map(|body| body.errcode.clone()),
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

fn section_block(title: &'static str, content: Element) -> Element {
    rsx! {
        div { class: "rounded-lg border p-4 space-y-3",
            h2 { class: "text-base font-semibold", "{title}" }
            {content}
        }
    }
}

fn detail_row(label: &'static str, value: &str) -> Element {
    rsx! {
        div { class: "flex items-start justify-between gap-4 border-b pb-2 last:border-b-0 last:pb-0",
            span { class: "text-sm text-muted-foreground", "{label}" }
            span { class: "text-sm text-right break-all", "{value}" }
        }
    }
}

fn bool_label(value: bool) -> &'static str {
    if value {
        "Yes"
    } else {
        "No"
    }
}
