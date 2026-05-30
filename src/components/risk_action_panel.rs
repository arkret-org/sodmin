use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::dangerous_action_dialog::DangerousActionDialog;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::{Input, Label};

/// Render the risk-action state-machine panel.
///
/// Called from `AccountDetailPage` with borrowed references to avoid
/// duplicating the data fetch.  Maintains its own signals for
/// proposal/approve/execute workflow state.
pub fn risk_action_panel(
    account_id: &str,
    current: &coauth::CoauthAccountRiskActionCurrentState,
    history: &[coauth::CoauthAccountRiskActionHistoryEntry],
    hook: &coauth::CoauthRiskActionHook,
    bridge: &coauth::CoauthAdminBridgeDescribe,
) -> Element {
    let account_id = account_id.to_string();

    let mut action_status = use_signal(String::new);
    let mut last_proposal = use_signal(|| Option::<coauth::CoauthAccountRiskActionProposal>::None);
    let mut last_approval = use_signal(|| Option::<coauth::CoauthAccountRiskActionApproval>::None);
    let mut proposal_reason = use_signal(String::new);
    let mut proposal_ticket = use_signal(String::new);
    let mut pending_proposal_action = use_signal::<Option<String>>(|| None);

    // State machine gating
    let allowed_transitions = &current.allowed_next_transitions;
    let lifecycle_state = current.lifecycle_state.as_str();
    let is_idle = lifecycle_state.is_empty() || lifecycle_state == "idle";
    let allowed_contains = |needle: &str| -> bool {
        allowed_transitions
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(needle))
    };
    let can_submit = is_idle
        || allowed_contains("submit")
        || allowed_contains("propose")
        || allowed_contains("queue");
    let can_approve = allowed_contains("approve");
    let approval_signing_available = risk_action_approval_signing_available();
    let can_execute = allowed_contains("execute");
    let allowed_summary = if allowed_transitions.is_empty() {
        "(none — backend has not exposed any next transition)".to_string()
    } else {
        allowed_transitions.join(", ")
    };
    let proposal_reason_ready = !proposal_reason.read().trim().is_empty();
    let proposal_ticket_ready = !proposal_ticket.read().trim().is_empty();
    let can_queue_proposal = can_submit && proposal_reason_ready && proposal_ticket_ready;

    rsx! {
        div { class: "rounded-lg border p-4 space-y-4",
            if !action_status().is_empty() {
                div { class: "rounded-md border p-3 text-sm text-muted-foreground",
                    "{action_status}"
                }
            }

            h2 { class: "text-base font-semibold", "Current Risk Action State" }
            if current.lifecycle_state == "idle" {
                p { class: "text-sm text-muted-foreground",
                    "{current.todo.clone().unwrap_or_else(|| \"No risk-action state has been recorded for this account yet.\".to_string())}"
                }
            } else {
                {current_state_grid(current)}
            }

            h2 { class: "text-base font-semibold mt-4", "Risk Action Transition History" }
            if history.is_empty() {
                p { class: "text-sm text-muted-foreground",
                    "No persisted risk-action transition records are currently returned for this account."
                }
            } else {
                div { class: "space-y-3",
                    for entry in history.iter() {
                        {history_entry_card(entry)}
                    }
                }
            }

            h2 { class: "text-base font-semibold mt-4", "High-Risk Action Hook" }
            p { class: "text-sm text-muted-foreground",
                "Approval mode: "
                span { class: "font-mono", "{hook.approval_mode}" }
            }
            p { class: "text-sm text-muted-foreground",
                "Endpoint: "
                span { class: "font-mono", "{hook.endpoint}" }
            }
            p { class: "text-sm text-muted-foreground",
                "Current state template: "
                span { class: "font-mono", "{bridge.risk_action_current_path_template}" }
            }
            p { class: "text-sm text-muted-foreground",
                "History template: "
                span { class: "font-mono", "{bridge.risk_action_history_path_template}" }
            }
            p { class: "text-sm text-muted-foreground",
                "Approve template: "
                span { class: "font-mono", "{bridge.risk_action_approve_path_template}" }
            }
            p { class: "text-sm text-muted-foreground",
                "Execute template: "
                span { class: "font-mono", "{bridge.risk_action_execute_path_template}" }
            }
            p { class: "text-sm text-muted-foreground",
                "State store: "
                span { class: "font-mono", "{bridge.risk_action_state_store_kind}" }
            }
            // C34.2: the three example payloads are now typed shared
            // structures from `coauth_admin_types::bridge_admin` instead
            // of opaque `serde_json::Value`. Round-trip them through
            // `serde_json` so the rendered string keeps the same JSON
            // shape the SPA used to display.
            p { class: "text-sm text-muted-foreground",
                "Proposal example: "
                span { class: "font-mono", "{serde_json::to_string(&bridge.risk_action_examples.proposal_request).unwrap_or_default()}" }
            }
            p { class: "text-sm text-muted-foreground",
                "Approve example: "
                span { class: "font-mono", "{serde_json::to_string(&bridge.risk_action_examples.approve_request).unwrap_or_default()}" }
            }
            p { class: "text-sm text-muted-foreground",
                "Execute example: "
                span { class: "font-mono", "{serde_json::to_string(&bridge.risk_action_examples.execute_request).unwrap_or_default()}" }
            }
            p { class: "text-sm text-muted-foreground", "{hook.todo}" }

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

            div { class: "rounded-md border p-3 text-xs text-muted-foreground",
                div { class: "font-medium text-foreground", "Risk-action state machine" }
                div { "Lifecycle: " span { class: "font-mono", "{lifecycle_state}" } }
                div { "Allowed next: " span { class: "font-mono", "{allowed_summary}" } }
                div { class: "mt-1",
                    "Buttons that map to a transition not in the allowed list are disabled. Reason and ticket are required before a proposal can be queued."
                }
                if !approval_signing_available {
                    div { class: "mt-1 text-amber-700",
                        "Risk-action approval now requires a detached EdDSA JWS from the authenticated admin DID. sodmin does not hold that signing key yet, so approval is disabled fail-closed."
                    }
                }
            }

            div { class: "grid gap-3 rounded-md border p-3 md:grid-cols-2",
                div { class: "space-y-1",
                    Label { r#for: "risk-action-reason".to_string(), "Reason" }
                    Input {
                        id: "risk-action-reason".to_string(),
                        placeholder: "Human-reviewed reason for this account action".to_string(),
                        value: proposal_reason.read().clone(),
                        required: true,
                        oninput: move |evt: FormEvent| proposal_reason.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "risk-action-ticket".to_string(), "Ticket" }
                    Input {
                        id: "risk-action-ticket".to_string(),
                        placeholder: "SEC-1234 / support case / incident id".to_string(),
                        value: proposal_ticket.read().clone(),
                        required: true,
                        oninput: move |evt: FormEvent| proposal_ticket.set(evt.value()),
                    }
                }
            }

            div { class: "flex flex-wrap gap-2",
                for action_label in [("lock", "Queue lock proposal"), ("disable", "Queue disable proposal"), ("reset_recovery", "Queue recovery reset proposal"), ("erase", "Queue erase proposal")] {
                    Button {
                        key: "{action_label.0}",
                        variant: ButtonVariant::Outline,
                        disabled: !can_queue_proposal,
                        onclick: {
                            let action = action_label.0.to_string();
                            move |_| {
                                if can_queue_proposal {
                                    pending_proposal_action.set(Some(action.clone()));
                                }
                            }
                        },
                        "{action_label.1}"
                    }
                }
                if let Some(proposal) = last_proposal() {
                    Button {
                        variant: ButtonVariant::Secondary,
                        disabled: !can_approve || !approval_signing_available,
                        onclick: {
                            let account_id = account_id.clone();
                            move |_| {
                                let account_id = account_id.clone();
                                let proposal = proposal.clone();
                                spawn(async move {
                                    let draft = build_risk_action_approval_draft(&proposal);
                                    match coauth::approve_account_risk_action(&account_id, &proposal.proposal_id, &draft).await {
                                        Ok(approval) => {
                                            last_approval.set(Some(approval.clone()));
                                            action_status.set(format_risk_action_approval_status(&approval));
                                        }
                                        Err(error) => action_status.set(format!("Approval failed: {}", error.message)),
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
                        disabled: !can_execute,
                        onclick: {
                            let account_id = account_id.clone();
                            move |_| {
                                let account_id = account_id.clone();
                                let approval = approval.clone();
                                spawn(async move {
                                    let draft = build_risk_action_execute_draft(&approval);
                                    match coauth::execute_account_risk_action(&account_id, &approval.proposal_id, &draft).await {
                                        Ok(execution) => action_status.set(format_risk_action_execute_status(&execution)),
                                        Err(error) => action_status.set(format!("Execute failed: {}", error.message)),
                                    }
                                });
                            }
                        },
                        "Execute approved action"
                    }
                }
            }

            if let Some(action) = pending_proposal_action() {
                {
                    let phrase = risk_action_phrase(&action);
                    let description = format!(
                        "Queue `{}` for account `{}` with the supplied reason and ticket. Type `{}` to confirm.",
                        action, account_id, phrase
                    );
                    rsx! {
                        DangerousActionDialog {
                            open: true,
                            title: format!("Queue risk action: {action}"),
                            description,
                            confirmation_phrase: phrase,
                            confirm_text: "Queue proposal".to_string(),
                            on_cancel: move |_| pending_proposal_action.set(None),
                            on_confirm: {
                                let account_id = account_id.clone();
                                let action = action.clone();
                                move |_| {
                                    let account_id = account_id.clone();
                                    let action = action.clone();
                                    let reason = proposal_reason.read().trim().to_string();
                                    let ticket = proposal_ticket.read().trim().to_string();
                                    if reason.is_empty() || ticket.is_empty() {
                                        action_status.set("Reason and ticket are required before queuing a risk-action proposal.".to_string());
                                        pending_proposal_action.set(None);
                                        return;
                                    }
                                    pending_proposal_action.set(None);
                                    spawn(async move {
                                        let draft = build_risk_action_draft(&action, reason, ticket);
                                        match coauth::submit_account_risk_action(&account_id, &draft).await {
                                            Ok(proposal) => {
                                                last_proposal.set(Some(proposal.clone()));
                                                last_approval.set(None);
                                                action_status.set(format_risk_action_status(&proposal));
                                            }
                                            Err(error) => action_status.set(format!("{} proposal failed: {}", action, error.message)),
                                        }
                                    });
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

fn current_state_grid(current: &coauth::CoauthAccountRiskActionCurrentState) -> Element {
    let proposal_id = current
        .proposal_id
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let state_record_id = current
        .state_record_id
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let action = current.action.clone().unwrap_or_else(|| "-".to_string());
    let last_operation = current
        .last_operation
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let transition_kind = current
        .transition_kind
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let previous_state = current
        .previous_state
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let state_revision = current
        .state_revision
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    let allowed_next = if current.allowed_next_transitions.is_empty() {
        "-".to_string()
    } else {
        current.allowed_next_transitions.join(", ")
    };
    let ticket = current.ticket.clone().unwrap_or_else(|| "-".to_string());
    let recorded_at = current
        .recorded_at
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| "-".to_string());
    let recorded_by = current
        .recorded_by
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let recorded_by_handle = current
        .recorded_by_handle
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let execution_endpoint = current
        .execution_endpoint
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let mutation_endpoint = current
        .mutation_endpoint
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let state_store_kind = current.state_store_kind.clone();
    let todo = current.todo.clone().unwrap_or_else(|| "-".to_string());

    rsx! {
        div { class: "grid gap-2 text-sm md:grid-cols-2",
            {detail_row("Lifecycle State", &current.lifecycle_state)}
            {detail_row("State Record ID", &state_record_id)}
            {detail_row("Proposal ID", &proposal_id)}
            {detail_row("Action", &action)}
            {detail_row("Last Operation", &last_operation)}
            {detail_row("Transition Kind", &transition_kind)}
            {detail_row("Previous State", &previous_state)}
            {detail_row("State Revision", &state_revision)}
            {detail_row("Allowed Next", &allowed_next)}
            {detail_row("Ticket", &ticket)}
            {detail_row("Recorded At", &recorded_at)}
            {detail_row("Recorded By", &recorded_by)}
            {detail_row("Recorded By Handle", &recorded_by_handle)}
            {detail_row("Execution Endpoint", &execution_endpoint)}
            {detail_row("Mutation Endpoint", &mutation_endpoint)}
            {detail_row("State Store", &state_store_kind)}
            {detail_row("Backend Notes", &todo)}
        }
    }
}

fn history_entry_card(entry: &coauth::CoauthAccountRiskActionHistoryEntry) -> Element {
    let state_record_id = entry
        .state_record_id
        .clone()
        .unwrap_or_else(|| "missing".to_string());
    let proposal_id = entry
        .proposal_id
        .clone()
        .unwrap_or_else(|| "missing".to_string());
    let action = entry
        .action
        .clone()
        .unwrap_or_else(|| "missing".to_string());
    let previous_state = entry
        .previous_state
        .clone()
        .unwrap_or_else(|| "missing".to_string());
    let state_revision = entry
        .state_revision
        .map(|v| v.to_string())
        .unwrap_or_else(|| "missing".to_string());
    let recorded_at = entry
        .recorded_at
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| "missing".to_string());

    rsx! {
        div { class: "rounded-md border p-3 space-y-1 text-sm text-muted-foreground",
            div { "State Record ID: " span { class: "font-mono", "{state_record_id}" } }
            div { "Proposal ID: " span { class: "font-mono", "{proposal_id}" } }
            div { "Action: " span { class: "font-mono", "{action}" } }
            div { "Transition: " span { class: "font-mono", "{entry.transition_kind}" } }
            div { "Previous State: " span { class: "font-mono", "{previous_state}" } }
            div { "Next State: " span { class: "font-mono", "{entry.next_state}" } }
            div { "State Revision: " span { class: "font-mono", "{state_revision}" } }
            div { "Ticket: " span { class: "font-mono", "{entry.ticket.as_deref().unwrap_or(\"missing\")}" } }
            div { "Recorded At: " span { class: "font-mono", "{recorded_at}" } }
            div { "Recorded By: " span { class: "font-mono", "{entry.recorded_by.as_deref().unwrap_or(\"missing\")}" } }
            div { "Recorded By Handle: " span { class: "font-mono", "{entry.recorded_by_handle.as_deref().unwrap_or(\"missing\")}" } }
            div { "Execution Endpoint: " span { class: "font-mono", "{entry.execution_endpoint.as_deref().unwrap_or(\"missing\")}" } }
            div { "Mutation Endpoint: " span { class: "font-mono", "{entry.mutation_endpoint.as_deref().unwrap_or(\"missing\")}" } }
            div { "Approval Note: " span { class: "font-mono", "{entry.approval_note.as_deref().unwrap_or(\"missing\")}" } }
            div { "Execution Note: " span { class: "font-mono", "{entry.execution_note.as_deref().unwrap_or(\"missing\")}" } }
            div { "State Store: " span { class: "font-mono", "{entry.state_store_kind}" } }
        }
    }
}

fn detail_row(label: &str, value: &str) -> Element {
    rsx! {
        div { class: "flex items-start justify-between gap-4 border-b pb-2 last:border-b-0 last:pb-0",
            span { class: "text-sm text-muted-foreground", "{label}" }
            span { class: "text-sm text-right break-all", "{value}" }
        }
    }
}

fn build_risk_action_draft(
    action: &str,
    reason: String,
    ticket: String,
) -> coauth::CoauthAccountRiskActionDraft {
    coauth::CoauthAccountRiskActionDraft {
        action: action.to_string(),
        reason: Some(reason),
        ticket: Some(ticket),
        approved_by: None,
    }
}

fn build_risk_action_approval_draft(
    proposal: &coauth::CoauthAccountRiskActionProposal,
) -> coauth::CoauthAccountRiskActionApprovalDraft {
    coauth::CoauthAccountRiskActionApprovalDraft {
        action: proposal.action.clone(),
        ticket: proposal.ticket.clone(),
        approved_by: None,
        approval_note: Some(format!(
            "sodmin approval for proposal {} action {}",
            proposal.proposal_id, proposal.action
        )),
        approval_proof_jws: String::new(),
    }
}

fn risk_action_approval_signing_available() -> bool {
    // coauth requires a detached EdDSA JWS over the approval transcript.
    // sodmin currently authenticates with a bearer token and does not possess
    // the admin DID private key, so the UI must not submit unverifiable
    // approvals.
    false
}

fn build_risk_action_execute_draft(
    approval: &coauth::CoauthAccountRiskActionApproval,
) -> coauth::CoauthAccountRiskActionExecuteDraft {
    coauth::CoauthAccountRiskActionExecuteDraft {
        action: approval.action.clone(),
        ticket: approval.ticket.clone(),
        execution_note: Some(format!(
            "sodmin execute proposal {} action {}",
            approval.proposal_id, approval.action
        )),
    }
}

fn format_risk_action_status(proposal: &coauth::CoauthAccountRiskActionProposal) -> String {
    format!(
        "Queued risk-action proposal.\nstate_record_id={}\nproposal_id={}\naction={}\nproposal_state={}\nstate_revision={}\ntransition_kind={}\napproval_mode={}\nexecution_endpoint={}\nrequested_at={}\nrequested_by={}\nrequested_by_handle={}\nticket={}\napproved_by={}\n\n{}",
        proposal.state_record_id,
        proposal.proposal_id,
        proposal.action,
        proposal.proposal_state,
        proposal.state_revision,
        proposal.transition_kind,
        proposal.approval_mode,
        proposal.execution_endpoint,
        proposal
            .requested_at
            .map(|dt| dt.to_rfc3339())
            .as_deref()
            .unwrap_or("missing"),
        proposal.requested_by.as_deref().unwrap_or("missing"),
        proposal.requested_by_handle.as_deref().unwrap_or("missing"),
        proposal.ticket.as_deref().unwrap_or("missing"),
        proposal.approved_by.as_deref().unwrap_or("pending"),
        proposal.todo,
    )
}

fn risk_action_phrase(action: &str) -> String {
    format!("QUEUE {}", action.to_ascii_uppercase())
}

fn format_risk_action_approval_status(
    approval: &coauth::CoauthAccountRiskActionApproval,
) -> String {
    format!(
        "Approved risk-action proposal.\nstate_record_id={}\nproposal_id={}\naction={}\napproval_state={}\nstate_revision={}\ntransition_kind={}\napproved_at={}\napproved_by={}\napproved_by_handle={}\nexecution_endpoint={}\napproval_note={}\n\n{}",
        approval.state_record_id,
        approval.proposal_id,
        approval.action,
        approval.approval_state,
        approval.state_revision,
        approval.transition_kind,
        approval
            .approved_at
            .map(|dt| dt.to_rfc3339())
            .as_deref()
            .unwrap_or("missing"),
        approval.approved_by.as_deref().unwrap_or("missing"),
        approval.approved_by_handle.as_deref().unwrap_or("missing"),
        approval.execution_endpoint,
        approval.approval_note.as_deref().unwrap_or("missing"),
        approval.todo,
    )
}

fn format_risk_action_execute_status(execution: &coauth::CoauthAccountRiskActionExecute) -> String {
    format!(
        "Executed risk-action.\nstate_record_id={}\nproposal_id={}\naction={}\nexecution_state={}\nstate_revision={}\ntransition_kind={}\nexecuted_at={}\nexecution_mode={}\nmutation_endpoint={}\nexecution_note={}\n\n{}",
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
