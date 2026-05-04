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
    let mut action_status = use_signal(String::new);
    let mut last_proposal = use_signal(|| Option::<coauth::CoauthAccountRiskActionProposal>::None);
    let mut last_approval = use_signal(|| Option::<coauth::CoauthAccountRiskActionApproval>::None);
    let mut data = use_resource(move || async move {
        coauth::get_account_detail(&account_id_for_resource).await
    });

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
            if !action_status().is_empty() {
                div { class: "rounded-lg border p-4 text-sm text-muted-foreground",
                    "{action_status}"
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

                        section_block(
                            "Current Risk Action State",
                            if detail.risk_action_current.lifecycle_state == "idle" {
                                rsx! {
                                    p { class: "text-sm text-muted-foreground",
                                        "{detail.risk_action_current.todo.clone().unwrap_or_else(|| \"No risk-action scaffold state has been recorded for this account yet.\".to_string())}"
                                    }
                                }
                            } else {
                                let proposal_id = detail.risk_action_current.proposal_id.clone().unwrap_or_else(|| "-".to_string());
                                let state_record_id = detail.risk_action_current.state_record_id.clone().unwrap_or_else(|| "-".to_string());
                                let action = detail.risk_action_current.action.clone().unwrap_or_else(|| "-".to_string());
                                let last_operation = detail.risk_action_current.last_operation.clone().unwrap_or_else(|| "-".to_string());
                                let transition_kind = detail.risk_action_current.transition_kind.clone().unwrap_or_else(|| "-".to_string());
                                let previous_state = detail.risk_action_current.previous_state.clone().unwrap_or_else(|| "-".to_string());
                                let state_revision = detail.risk_action_current.state_revision.map(|value| value.to_string()).unwrap_or_else(|| "-".to_string());
                                let allowed_next = if detail.risk_action_current.allowed_next_transitions.is_empty() {
                                    "-".to_string()
                                } else {
                                    detail.risk_action_current.allowed_next_transitions.join(", ")
                                };
                                let ticket = detail.risk_action_current.ticket.clone().unwrap_or_else(|| "-".to_string());
                                let recorded_at = detail.risk_action_current.recorded_at.clone().unwrap_or_else(|| "-".to_string());
                                let recorded_by = detail.risk_action_current.recorded_by.clone().unwrap_or_else(|| "-".to_string());
                                let recorded_by_username = detail.risk_action_current.recorded_by_username.clone().unwrap_or_else(|| "-".to_string());
                                let execution_endpoint = detail.risk_action_current.execution_endpoint.clone().unwrap_or_else(|| "-".to_string());
                                let mutation_endpoint = detail.risk_action_current.mutation_endpoint.clone().unwrap_or_else(|| "-".to_string());
                                let state_store_kind = detail.risk_action_current.state_store_kind.clone();
                                let todo = detail.risk_action_current.todo.clone().unwrap_or_else(|| "-".to_string());
                                rsx! {
                                    div { class: "grid gap-2 text-sm md:grid-cols-2",
                                        detail_row("Lifecycle State", &detail.risk_action_current.lifecycle_state)
                                        detail_row("State Record ID", &state_record_id)
                                        detail_row("Proposal ID", &proposal_id)
                                        detail_row("Action", &action)
                                        detail_row("Last Operation", &last_operation)
                                        detail_row("Transition Kind", &transition_kind)
                                        detail_row("Previous State", &previous_state)
                                        detail_row("State Revision", &state_revision)
                                        detail_row("Allowed Next", &allowed_next)
                                        detail_row("Ticket", &ticket)
                                        detail_row("Recorded At", &recorded_at)
                                        detail_row("Recorded By", &recorded_by)
                                        detail_row("Recorded By Username", &recorded_by_username)
                                        detail_row("Execution Endpoint", &execution_endpoint)
                                        detail_row("Mutation Endpoint", &mutation_endpoint)
                                        detail_row("State Store", &state_store_kind)
                                        detail_row("TODO", &todo)
                                    }
                                }
                            },
                        )

                        section_block(
                            "Risk Action Transition History",
                            if detail.risk_action_history.is_empty() {
                                rsx! {
                                    p { class: "text-sm text-muted-foreground",
                                        "No persisted risk-action transition records are currently returned for this account."
                                    }
                                }
                            } else {
                                rsx! {
                                    div { class: "space-y-3",
                                        for entry in &detail.risk_action_history {
                                            let state_record_id = entry.state_record_id.clone().unwrap_or_else(|| "missing".to_string());
                                            let proposal_id = entry.proposal_id.clone().unwrap_or_else(|| "missing".to_string());
                                            let action = entry.action.clone().unwrap_or_else(|| "missing".to_string());
                                            let previous_state = entry.previous_state.clone().unwrap_or_else(|| "missing".to_string());
                                            let state_revision = entry.state_revision.map(|value| value.to_string()).unwrap_or_else(|| "missing".to_string());
                                            div { class: "rounded-md border p-3 space-y-1 text-sm text-muted-foreground",
                                                div { "State Record ID: " span { class: "font-mono", "{state_record_id}" } }
                                                div { "Proposal ID: " span { class: "font-mono", "{proposal_id}" } }
                                                div { "Action: " span { class: "font-mono", "{action}" } }
                                                div { "Transition: " span { class: "font-mono", "{entry.transition_kind}" } }
                                                div { "Previous State: " span { class: "font-mono", "{previous_state}" } }
                                                div { "Next State: " span { class: "font-mono", "{entry.next_state}" } }
                                                div { "State Revision: " span { class: "font-mono", "{state_revision}" } }
                                                div { "Ticket: " span { class: "font-mono", "{entry.ticket.as_deref().unwrap_or(\"missing\")}" } }
                                                div { "Recorded At: " span { class: "font-mono", "{entry.recorded_at.as_deref().unwrap_or(\"missing\")}" } }
                                                div { "Recorded By: " span { class: "font-mono", "{entry.recorded_by.as_deref().unwrap_or(\"missing\")}" } }
                                                div { "Recorded By Username: " span { class: "font-mono", "{entry.recorded_by_username.as_deref().unwrap_or(\"missing\")}" } }
                                                div { "Execution Endpoint: " span { class: "font-mono", "{entry.execution_endpoint.as_deref().unwrap_or(\"missing\")}" } }
                                                div { "Mutation Endpoint: " span { class: "font-mono", "{entry.mutation_endpoint.as_deref().unwrap_or(\"missing\")}" } }
                                                div { "Approval Note: " span { class: "font-mono", "{entry.approval_note.as_deref().unwrap_or(\"missing\")}" } }
                                                div { "Execution Note: " span { class: "font-mono", "{entry.execution_note.as_deref().unwrap_or(\"missing\")}" } }
                                                div { "State Store: " span { class: "font-mono", "{entry.state_store_kind}" } }
                                            }
                                        }
                                    }
                                }
                            },
                        )

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
                            h2 { class: "text-base font-semibold", "Recovery Bridge Contract" }
                            p { class: "text-sm text-muted-foreground",
                                "Contract: "
                                span { class: "font-mono", "{detail.recovery_bridge.contract}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Version: "
                                span { class: "font-mono", "{detail.recovery_bridge.version}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Recovery Start: "
                                span { class: "font-mono", "{detail.recovery_bridge.recovery_start_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Recovery Status: "
                                span { class: "font-mono", "{detail.recovery_bridge.recovery_status_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Recovery Resend: "
                                span { class: "font-mono", "{detail.recovery_bridge.recovery_resend_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Key Backup Base: "
                                span { class: "font-mono", "{detail.recovery_bridge.key_backup_rest_base}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Key Backup Schema: "
                                span { class: "font-mono", "{detail.recovery_bridge.key_backup_schema}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Device Message Schema: "
                                span { class: "font-mono", "{detail.recovery_bridge.device_message_schema}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Recovery Contract Stack: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_recovery_contract_stack_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Device Messages Describe: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_device_messages_describe_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Key Backups Describe: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_key_backups_describe_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore State Describe: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_state_describe_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore State Export: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_state_export_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore State Import: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_state_import_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Start: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_start_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Describe: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_describe_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Ticket: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_ticket_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Ticket Advance: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_ticket_advance_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Approval Status: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_approval_status_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Approval Submit: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_approval_submit_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Executor Status: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_executor_status_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Executor Enqueue: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_executor_enqueue_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Executor Start: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_executor_start_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Executor Complete: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_executor_complete_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Result: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_result_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Receipt: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_receipt_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Materialized Device Handoff: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_materialized_device_handoff_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Restore Bundle: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_restore_bundle_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Authz Describe: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_authz_describe_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Authz Check: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_authz_check_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Policy Describe: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_policy_describe_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Policy Collection: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_policy_collection_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Principal Policy Item: "
                                span { class: "font-mono", "{detail.recovery_bridge.principal_policy_item_path}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Verification Kinds: "
                                span { class: "font-mono", "{detail.recovery_bridge.verification_event_kinds.join(\", \")}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Recovery Modes: "
                                span { class: "font-mono", "{detail.recovery_bridge.recovery_modes.join(\", \")}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Backup Example: "
                                span { class: "font-mono", "{detail.recovery_bridge.example_backup_payload}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Recovery Restore Example: "
                                span { class: "font-mono", "{detail.recovery_bridge.recovery_restore_examples}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Recovery Authz Example: "
                                span { class: "font-mono", "{detail.recovery_bridge.recovery_authz_examples}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "{detail.recovery_bridge.todos.join(\" \")}"
                            }
                            h2 { class: "text-base font-semibold", "High-Risk Action Hook" }
                            p { class: "text-sm text-muted-foreground",
                                "Approval mode: "
                                span { class: "font-mono", "{detail.risk_action_hook.approval_mode}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Endpoint: "
                                span { class: "font-mono", "{detail.risk_action_hook.endpoint}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Current state template: "
                                span { class: "font-mono", "{detail.admin_bridge.risk_action_current_path_template}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "History template: "
                                span { class: "font-mono", "{detail.admin_bridge.risk_action_history_path_template}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Approve template: "
                                span { class: "font-mono", "{detail.admin_bridge.risk_action_approve_path_template}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Execute template: "
                                span { class: "font-mono", "{detail.admin_bridge.risk_action_execute_path_template}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "State store: "
                                span { class: "font-mono", "{detail.admin_bridge.risk_action_state_store_kind}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Proposal example: "
                                span { class: "font-mono", "{detail.admin_bridge.risk_action_examples.proposal_request}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Approve example: "
                                span { class: "font-mono", "{detail.admin_bridge.risk_action_examples.approve_request}" }
                            }
                            p { class: "text-sm text-muted-foreground",
                                "Execute example: "
                                span { class: "font-mono", "{detail.admin_bridge.risk_action_examples.execute_request}" }
                            }
                            p { class: "text-sm text-muted-foreground", "{detail.risk_action_hook.todo}" }
                            if let Some(proposal) = last_proposal() {
                                div { class: "rounded-md border p-3 space-y-1 text-sm text-muted-foreground",
                                    div { "Last proposal: " span { class: "font-mono", "{proposal.proposal_id}" } }
                                    div { "State record: " span { class: "font-mono", "{proposal.state_record_id}" } }
                                    div { "Action: " span { class: "font-mono", "{proposal.action}" } }
                                    div { "State: " span { class: "font-mono", "{proposal.proposal_state}" } }
                                    div { "Revision: " span { class: "font-mono", "{proposal.state_revision}" } }
                                    div { "Execution endpoint: " span { class: "font-mono", "{proposal.execution_endpoint}" } }
                                }
                            }
                            div { class: "flex flex-wrap gap-2",
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: {
                                        let account_id = account_id.clone();
                                        move |_| {
                                            spawn(async move {
                                                let draft = build_risk_action_draft("lock", &account_id);
                                                match coauth::submit_account_risk_action(&account_id, &draft).await {
                                                    Ok(proposal) => {
                                                        last_proposal.set(Some(proposal.clone()));
                                                        last_approval.set(None);
                                                        action_status.set(format_risk_action_status(&proposal));
                                                    }
                                                    Err(error) => action_status.set(format!("Lock proposal failed: {}", error.message)),
                                                }
                                            });
                                        }
                                    },
                                    "Queue lock proposal"
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: {
                                        let account_id = account_id.clone();
                                        move |_| {
                                            spawn(async move {
                                                let draft = build_risk_action_draft("disable", &account_id);
                                                match coauth::submit_account_risk_action(&account_id, &draft).await {
                                                    Ok(proposal) => {
                                                        last_proposal.set(Some(proposal.clone()));
                                                        last_approval.set(None);
                                                        action_status.set(format_risk_action_status(&proposal));
                                                    }
                                                    Err(error) => action_status.set(format!("Disable proposal failed: {}", error.message)),
                                                }
                                            });
                                        }
                                    },
                                    "Queue disable proposal"
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: {
                                        let account_id = account_id.clone();
                                        move |_| {
                                            spawn(async move {
                                                let draft = build_risk_action_draft("reset_recovery", &account_id);
                                                match coauth::submit_account_risk_action(&account_id, &draft).await {
                                                    Ok(proposal) => {
                                                        last_proposal.set(Some(proposal.clone()));
                                                        last_approval.set(None);
                                                        action_status.set(format_risk_action_status(&proposal));
                                                    }
                                                    Err(error) => action_status.set(format!("Recovery reset proposal failed: {}", error.message)),
                                                }
                                            });
                                        }
                                    },
                                    "Queue recovery reset proposal"
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: {
                                        let account_id = account_id.clone();
                                        move |_| {
                                            spawn(async move {
                                                let draft = build_risk_action_draft("erase", &account_id);
                                                match coauth::submit_account_risk_action(&account_id, &draft).await {
                                                    Ok(proposal) => {
                                                        last_proposal.set(Some(proposal.clone()));
                                                        last_approval.set(None);
                                                        action_status.set(format_risk_action_status(&proposal));
                                                    }
                                                    Err(error) => action_status.set(format!("Erase proposal failed: {}", error.message)),
                                                }
                                            });
                                        }
                                    },
                                    "Queue erase proposal"
                                }
                                if let Some(proposal) = last_proposal() {
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        onclick: {
                                            let account_id = account_id.clone();
                                            move |_| {
                                                let proposal = proposal.clone();
                                                spawn(async move {
                                                    let draft = build_risk_action_approval_draft(&proposal);
                                                    match coauth::approve_account_risk_action(
                                                        &account_id,
                                                        &proposal.proposal_id,
                                                        &draft,
                                                    )
                                                    .await
                                                    {
                                                        Ok(approval) => {
                                                            last_approval.set(Some(approval.clone()));
                                                            action_status.set(format_risk_action_approval_status(&approval));
                                                        }
                                                        Err(error) => action_status.set(format!("Approval scaffold failed: {}", error.message)),
                                                    }
                                                });
                                            }
                                        },
                                        "Approve last proposal"
                                    }
                                }
                                if let Some(approval) = last_approval() {
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        onclick: {
                                            let account_id = account_id.clone();
                                            move |_| {
                                                let approval = approval.clone();
                                                spawn(async move {
                                                    let draft = build_risk_action_execute_draft(&approval);
                                                    match coauth::execute_account_risk_action(
                                                        &account_id,
                                                        &approval.proposal_id,
                                                        &draft,
                                                    )
                                                    .await
                                                    {
                                                        Ok(execution) => action_status.set(format_risk_action_execute_status(&execution)),
                                                        Err(error) => action_status.set(format!("Execute scaffold failed: {}", error.message)),
                                                    }
                                                });
                                            }
                                        },
                                        "Execute approved scaffold"
                                    }
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

fn build_risk_action_draft(
    action: &str,
    account_id: &str,
) -> coauth::CoauthAccountRiskActionDraft {
    coauth::CoauthAccountRiskActionDraft {
        action: action.to_string(),
        reason: Some(format!(
            "sodmin scaffold proposal for account {} action {}",
            account_id, action
        )),
        ticket: Some(format!("TODO-{}-{}", action, account_id)),
        approved_by: None,
    }
}

fn build_risk_action_approval_draft(
    proposal: &coauth::CoauthAccountRiskActionProposal,
) -> coauth::CoauthAccountRiskActionApprovalDraft {
    coauth::CoauthAccountRiskActionApprovalDraft {
        action: proposal.action.clone(),
        ticket: proposal.ticket.clone(),
        approved_by: proposal.approved_by.clone(),
        approval_note: Some(format!(
            "sodmin scaffold approval for proposal {} action {}",
            proposal.proposal_id, proposal.action
        )),
    }
}

fn build_risk_action_execute_draft(
    approval: &coauth::CoauthAccountRiskActionApproval,
) -> coauth::CoauthAccountRiskActionExecuteDraft {
    coauth::CoauthAccountRiskActionExecuteDraft {
        action: approval.action.clone(),
        ticket: approval.ticket.clone(),
        execution_note: Some(format!(
            "sodmin execute scaffold for proposal {} action {}",
            approval.proposal_id, approval.action
        )),
    }
}

fn format_risk_action_status(proposal: &coauth::CoauthAccountRiskActionProposal) -> String {
    format!(
        "Queued risk-action proposal.\nstate_record_id={}\nproposal_id={}\naction={}\nproposal_state={}\nstate_revision={}\ntransition_kind={}\napproval_mode={}\nexecution_endpoint={}\nrequested_at={}\nrequested_by={}\nrequested_by_username={}\nticket={}\napproved_by={}\n\n{}",
        proposal.state_record_id,
        proposal.proposal_id,
        proposal.action,
        proposal.proposal_state,
        proposal.state_revision,
        proposal.transition_kind,
        proposal.approval_mode,
        proposal.execution_endpoint,
        proposal.requested_at.as_deref().unwrap_or("missing"),
        proposal.requested_by.as_deref().unwrap_or("missing"),
        proposal
            .requested_by_username
            .as_deref()
            .unwrap_or("missing"),
        proposal.ticket.as_deref().unwrap_or("missing"),
        proposal.approved_by.as_deref().unwrap_or("pending"),
        proposal.todo,
    )
}

fn format_risk_action_approval_status(
    approval: &coauth::CoauthAccountRiskActionApproval,
) -> String {
    format!(
        "Approved risk-action proposal.\nstate_record_id={}\nproposal_id={}\naction={}\napproval_state={}\nstate_revision={}\ntransition_kind={}\napproved_at={}\napproved_by={}\napproved_by_username={}\nexecution_endpoint={}\napproval_note={}\n\n{}",
        approval.state_record_id,
        approval.proposal_id,
        approval.action,
        approval.approval_state,
        approval.state_revision,
        approval.transition_kind,
        approval.approved_at.as_deref().unwrap_or("missing"),
        approval.approved_by.as_deref().unwrap_or("missing"),
        approval
            .approved_by_username
            .as_deref()
            .unwrap_or("missing"),
        approval.execution_endpoint,
        approval.approval_note.as_deref().unwrap_or("missing"),
        approval.todo,
    )
}

fn format_risk_action_execute_status(
    execution: &coauth::CoauthAccountRiskActionExecute,
) -> String {
    format!(
        "Executed risk-action scaffold.\nstate_record_id={}\nproposal_id={}\naction={}\nexecution_state={}\nstate_revision={}\ntransition_kind={}\nexecuted_at={}\nexecution_mode={}\nmutation_endpoint={}\nexecution_note={}\n\n{}",
        execution.state_record_id,
        execution.proposal_id,
        execution.action,
        execution.execution_state,
        execution.state_revision,
        execution.transition_kind,
        execution.executed_at.as_deref().unwrap_or("missing"),
        execution.execution_mode,
        execution.mutation_endpoint,
        execution.execution_note.as_deref().unwrap_or("missing"),
        execution.todo,
    )
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
