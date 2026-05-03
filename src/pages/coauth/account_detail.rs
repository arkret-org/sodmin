use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::router::Route;

#[component]
pub fn AccountDetailPage(account_id: String) -> Element {
    let account_id_for_resource = account_id.clone();
    let mut data = use_resource(move || async move {
        coauth::get_account_detail(&account_id_for_resource).await
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "coauth Account Detail".to_string(),
                description: "Account-first admin detail view backed by coauth account and DID-binding endpoints. Claims and grant inventory remain partial until backend coverage expands.".to_string(),
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

                    rsx! {
                        div { class: "rounded-lg border p-4 space-y-2",
                            div { class: "text-lg font-semibold", "{display_name}" }
                            div { class: "text-sm text-muted-foreground",
                                "Bridge status: "
                                span { class: "font-mono", "{summary.bridge_status}" }
                            }
                            div { class: "text-sm text-muted-foreground",
                                "This panel is backed by coauth /accounts and /accounts/{id}/dids. TODO(contract): fill claims, session-grant inventory, and approval metadata from dedicated backend fields once they exist."
                            }
                        }

                        div { class: "grid gap-4 md:grid-cols-2",
                            section_block(
                                "Account Summary",
                                rsx! {
                                    detail_row("Account ID", &summary.id)
                                    detail_row("Handle", &handle)
                                    detail_row("Email", &email)
                                    detail_row("Primary DID", &primary_did)
                                },
                            )
                            section_block(
                                "Lifecycle",
                                rsx! {
                                    detail_row("Status", lifecycle)
                                    detail_row("Locked", bool_label(summary.is_locked))
                                    detail_row("Deactivated", bool_label(summary.is_deactivated))
                                    detail_row("Created At", &created_at)
                                    detail_row("Updated At", &updated_at)
                                },
                            )
                        }

                        section_block(
                            "Managed DID Bindings",
                            if detail.managed_dids.is_empty() {
                                rsx! {
                                    p { class: "text-sm text-muted-foreground",
                                        "No DID bindings are currently returned for this account."
                                    }
                                }
                            } else {
                                rsx! {
                                    ul { class: "space-y-2",
                                        for binding in detail.managed_dids.iter() {
                                            let method = binding.method.clone().unwrap_or_else(|| "-".to_string());
                                            let state = binding.state.clone().unwrap_or_else(|| "-".to_string());
                                            let verified_at = binding.last_verified_at.clone().unwrap_or_else(|| "-".to_string());
                                            li { class: "rounded-md border p-3",
                                                div { class: "font-mono text-sm", "{binding.did}" }
                                                div { class: "mt-2 grid gap-2 text-sm md:grid-cols-3",
                                                    detail_row("Method", &method)
                                                    detail_row("State", &state)
                                                    detail_row("Last Verified", &verified_at)
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                        )

                        section_block(
                            "Claims",
                            if detail.claims.is_empty() {
                                rsx! {
                                    p { class: "text-sm text-muted-foreground",
                                        "No claim material is exposed by the current coauth account admin contract."
                                    }
                                }
                            } else {
                                rsx! {
                                    ul { class: "space-y-2",
                                        for claim in detail.claims.iter() {
                                            let value = claim.value.clone().unwrap_or_else(|| "-".to_string());
                                            let state = claim.state.clone().unwrap_or_else(|| "-".to_string());
                                            let source = claim.source.clone().unwrap_or_else(|| "-".to_string());
                                            li { class: "rounded-md border p-3",
                                                div { class: "font-medium", "{claim.claim_type}" }
                                                div { class: "mt-2 grid gap-2 text-sm md:grid-cols-3",
                                                    detail_row("Value", &value)
                                                    detail_row("State", &state)
                                                    detail_row("Source", &source)
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                        )

                        section_block(
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
                                            let subject = grant.subject.clone().unwrap_or_else(|| "-".to_string());
                                            let scope = grant.scope.clone().unwrap_or_else(|| "-".to_string());
                                            let state = grant.state.clone().unwrap_or_else(|| "-".to_string());
                                            let issued_at = grant.issued_at.clone().unwrap_or_else(|| "-".to_string());
                                            li { class: "rounded-md border p-3",
                                                div { class: "font-mono text-sm", "{grant.grant_id}" }
                                                div { class: "mt-2 grid gap-2 text-sm md:grid-cols-2",
                                                    detail_row("Subject", &subject)
                                                    detail_row("Scope", &scope)
                                                    detail_row("State", &state)
                                                    detail_row("Issued At", &issued_at)
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                        )

                        div { class: "rounded-lg border p-4 space-y-4",
                            h2 { class: "text-base font-semibold", "High-Risk Action Hook" }
                            p { class: "text-sm text-muted-foreground",
                                "Approval mode: "
                                span { class: "font-mono", "{detail.risk_action_hook.approval_mode}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Endpoint: "
                                span { class: "font-mono", "{detail.risk_action_hook.endpoint}" }
                            }
                            p { class: "text-sm text-muted-foreground", "{detail.risk_action_hook.todo}" }
                            div { class: "flex flex-wrap gap-2",
                                Button {
                                    variant: ButtonVariant::Outline,
                                    disabled: true,
                                    "Create lock proposal"
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    disabled: true,
                                    "Create deactivate proposal"
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    disabled: true,
                                    "Create password reset proposal"
                                }
                            }
                        }
                    }
                }
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
