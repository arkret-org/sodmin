//! CXP-0008 / CXP-0009 personal-agent admin pages.
//!
//! Mounted at `/agents/personal` (list) and
//! `/agents/personal/:agent_id` (detail). Drives the 11 new soland
//! HTTP endpoints + the coauth `accountability_grant` mint via the
//! `cx.agent.manage` admin scope.
//!
//! TODO(P3-impl): rich form layout for the wizard steps, soft validation
//! of accountability-grant freshness windows, deep validators on
//! capability grant scopes, axe-core assertions for the destructive
//! deactivation flow.

use dioxus::prelude::*;

use crate::api::agents;
use crate::components::dangerous_action_dialog::DangerousActionDialog;
use crate::components::granted_capabilities_view::GrantedCapabilitiesView;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::Label;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::components::validated_input::{ValidatedInput, ValidationKind};
use crate::types::{AccountabilityGrantRequest, Agent, AgentProvisionRequest};

const PAGE_SIZE: u64 = 25;

// ── List view ──

#[component]
pub fn PersonalAgentList() -> Element {
    let page = use_signal(|| 1u64);
    let mut wizard_open = use_signal(|| false);

    let page_val = *page.read();
    let mut data =
        use_resource(
            move || async move { agents::list_personal_agents(page_val, PAGE_SIZE).await },
        );

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Personal agents".to_string(),
                description: "Controller-self native personal agents (CXP-0008)".to_string(),
                Button {
                    size: ButtonSize::Sm,
                    onclick: move |_| wizard_open.set(true),
                    "Provision new agent"
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { "Agent id" }
                                    TableHead { "Name" }
                                    TableHead { "Controller DID" }
                                    TableHead { "actor_kind" }
                                    TableHead { "Pairing" }
                                    TableHead { "Grant freshness" }
                                    TableHead { "Status" }
                                    TableHead { "Actions" }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { colspan: 99,
                                            class: "text-center text-muted-foreground py-8".to_string(),
                                            "No personal agents provisioned yet."
                                        }
                                    }
                                } else {
                                    for agent in resp.data.iter() {
                                        {render_list_row(agent)}
                                    }
                                }
                            }
                        }
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

            if *wizard_open.read() {
                ProvisionWizard {
                    on_close: move |_| {
                        wizard_open.set(false);
                        data.restart();
                    },
                }
            }
        }
    }
}

fn render_list_row(agent: &Agent) -> Element {
    let id = agent.id.clone();
    let name = agent.name.clone().unwrap_or_else(|| "-".into());
    let controller = agent.controller_did.clone().unwrap_or_else(|| "-".into());
    let actor_kind = agent.actor_kind.clone().unwrap_or_else(|| "agent".into());
    let pairing = agent.pairing_status.clone().unwrap_or_else(|| "-".into());
    let freshness = agent
        .accountability_grant_refreshed_at
        .clone()
        .unwrap_or_else(|| "-".into());
    let status = agent.status.clone().unwrap_or_else(|| "-".into());
    let detail_route_id = id.clone();
    let kind_badge_variant = match actor_kind.as_str() {
        "agent" => BadgeVariant::Default,
        "integration" => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    };

    rsx! {
        TableRow {
            TableCell { class: "font-mono text-xs".to_string(), "{id}" }
            TableCell { "{name}" }
            TableCell { class: "font-mono text-xs max-w-[180px] truncate".to_string(), "{controller}" }
            TableCell {
                Badge { variant: kind_badge_variant, "{actor_kind}" }
            }
            TableCell { "{pairing}" }
            TableCell { class: "text-xs text-muted-foreground".to_string(), "{freshness}" }
            TableCell { "{status}" }
            TableCell {
                Link {
                    to: crate::router::Route::PersonalAgentShow { agent_id: detail_route_id },
                    class: "text-primary hover:underline text-sm",
                    "Open"
                }
            }
        }
    }
}

// ── Detail view ──

#[derive(Props, Clone, PartialEq)]
pub struct PersonalAgentShowProps {
    pub agent_id: String,
}

#[component]
pub fn PersonalAgentShow(props: PersonalAgentShowProps) -> Element {
    let agent_id_for_resource = props.agent_id.clone();
    let mut data = use_resource(move || {
        let id = agent_id_for_resource.clone();
        async move { agents::get_personal_agent(&id).await }
    });

    let mut deactivate_open = use_signal(|| false);

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: format!("Agent {}", props.agent_id),
                description: "Personal agent detail (CXP-0008)".to_string(),
            }

            match &*data.read() {
                Some(Ok(agent)) => {
                    let agent_id_for_actions = props.agent_id.clone();
                    let agent_id_pause = agent_id_for_actions.clone();
                    let agent_id_resume = agent_id_for_actions.clone();
                    let agent_id_rotate = agent_id_for_actions.clone();
                    let agent_id_grant_attach = agent_id_for_actions.clone();
                    let agent_id_sidecar = agent_id_for_actions.clone();
                    let agent_id_deactivate = agent_id_for_actions.clone();
                    let actor_kind = agent.actor_kind.clone().unwrap_or_default();
                    let pairing = agent.pairing_status.clone().unwrap_or_else(|| "-".into());
                    let freshness = agent
                        .accountability_grant_refreshed_at
                        .clone()
                        .unwrap_or_else(|| "-".into());
                    let agent_keys = agent.agent_keys.clone();
                    let capabilities = agent.capabilities.clone();

                    rsx! {
                        div { class: "grid gap-4 md:grid-cols-2",
                            div { class: "rounded-md border p-4 space-y-2",
                                h3 { class: "font-medium", "Identity" }
                                p { class: "text-sm", "actor_kind: " span { class: "font-mono", "{actor_kind}" } }
                                p { class: "text-sm", "Pairing: {pairing}" }
                                p { class: "text-sm", "Accountability grant refreshed at: {freshness}" }
                            }
                            div { class: "rounded-md border p-4 space-y-2",
                                h3 { class: "font-medium", "Agent keys" }
                                if agent_keys.is_empty() {
                                    p { class: "text-sm text-muted-foreground", "No agent keys on record." }
                                } else {
                                    ul { class: "text-xs font-mono space-y-1",
                                        for key in agent_keys.iter() {
                                            li { "{key}" }
                                        }
                                    }
                                }
                            }
                            div { class: "rounded-md border p-4 space-y-2 md:col-span-2",
                                h3 { class: "font-medium", "Capability grants" }
                                // TODO(P3-impl): real grant editor with the
                                // 14 capability actions surfacing in a
                                // dropdown; for now we list active grants
                                // verbatim.
                                if capabilities.is_empty() {
                                    p { class: "text-sm text-muted-foreground", "No active grants." }
                                } else {
                                    ul { class: "text-xs space-y-1",
                                        for cap in capabilities.iter() {
                                            li { class: "font-mono", "{cap}" }
                                        }
                                    }
                                }
                            }
                        }

                        // Action buttons.
                        div { class: "flex flex-wrap gap-2 pt-4 border-t",
                            Button {
                                size: ButtonSize::Sm,
                                onclick: move |_| {
                                    let id = agent_id_pause.clone();
                                    spawn(async move {
                                        match agents::pause_personal_agent(&id).await {
                                            Ok(_) => show_toast("Paused", ToastVariant::Success),
                                            Err(e) => show_toast(&e.message, ToastVariant::Error),
                                        }
                                    });
                                },
                                "Pause"
                            }
                            Button {
                                size: ButtonSize::Sm,
                                variant: ButtonVariant::Secondary,
                                onclick: move |_| {
                                    let id = agent_id_resume.clone();
                                    spawn(async move {
                                        match agents::resume_personal_agent(&id).await {
                                            Ok(_) => show_toast("Resumed", ToastVariant::Success),
                                            Err(e) => show_toast(&e.message, ToastVariant::Error),
                                        }
                                    });
                                },
                                "Resume"
                            }
                            Button {
                                size: ButtonSize::Sm,
                                variant: ButtonVariant::Outline,
                                onclick: move |_| {
                                    let id = agent_id_rotate.clone();
                                    spawn(async move {
                                        match agents::rotate_personal_agent_key(&id).await {
                                            Ok(_) => show_toast("Key rotated", ToastVariant::Success),
                                            Err(e) => show_toast(&e.message, ToastVariant::Error),
                                        }
                                    });
                                },
                                "Rotate key"
                            }
                            Button {
                                size: ButtonSize::Sm,
                                variant: ButtonVariant::Outline,
                                onclick: move |_| {
                                    // TODO(P3-impl): real grant.attach form with
                                    // the 14 capability action dropdown +
                                    // optional scope. For now we hit the wire
                                    // with the meta-scope cx.agent.manage so
                                    // the contract is exercised.
                                    let id = agent_id_grant_attach.clone();
                                    spawn(async move {
                                        match agents::attach_personal_agent_grant(
                                            &id,
                                            "cx.agent.manage",
                                            None,
                                        ).await {
                                            Ok(_) => show_toast("Grant attached", ToastVariant::Success),
                                            Err(e) => show_toast(&e.message, ToastVariant::Error),
                                        }
                                    });
                                },
                                "Grant: attach (cx.agent.manage)"
                            }
                            Button {
                                size: ButtonSize::Sm,
                                variant: ButtonVariant::Outline,
                                onclick: move |_| {
                                    let id = agent_id_sidecar.clone();
                                    spawn(async move {
                                        match agents::ensure_sidecar_thread(&id).await {
                                            Ok(_) => show_toast("Sidecar thread ensured", ToastVariant::Success),
                                            Err(e) => show_toast(&e.message, ToastVariant::Error),
                                        }
                                    });
                                },
                                "Ensure sidecar thread"
                            }
                            Button {
                                size: ButtonSize::Sm,
                                variant: ButtonVariant::Destructive,
                                onclick: move |_| deactivate_open.set(true),
                                "Deactivate"
                            }
                        }

                        DangerousActionDialog {
                            open: *deactivate_open.read(),
                            title: "Deactivate personal agent".to_string(),
                            description: "Type DEACTIVATE to confirm. This revokes all agent keys, capabilities, and runtime endpoints.".to_string(),
                            confirmation_phrase: "DEACTIVATE".to_string(),
                            confirm_text: "Deactivate agent".to_string(),
                            on_cancel: move |_| deactivate_open.set(false),
                            on_confirm: move |_| {
                                let id = agent_id_deactivate.clone();
                                deactivate_open.set(false);
                                spawn(async move {
                                    match agents::deactivate_personal_agent(&id).await {
                                        Ok(_) => show_toast("Agent deactivated", ToastVariant::Success),
                                        Err(e) => show_toast(&e.message, ToastVariant::Error),
                                    }
                                });
                            },
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

// ── Provision wizard (3 steps) ──

#[derive(Props, Clone, PartialEq)]
pub struct ProvisionWizardProps {
    pub on_close: EventHandler<()>,
}

#[component]
fn ProvisionWizard(props: ProvisionWizardProps) -> Element {
    let mut step = use_signal(|| 1u8);
    let mut controller_did = use_signal(String::new);
    let mut display_name = use_signal(String::new);
    let mut agent_principal_id = use_signal(String::new);
    let mut error_msg = use_signal(String::new);

    let close = props.on_close;

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
            div { class: "bg-background rounded-md border shadow-lg p-6 w-full max-w-lg space-y-4",
                h2 { class: "text-lg font-semibold", "Provision personal agent (step {step})" }

                if !error_msg.read().is_empty() {
                    div { class: "rounded border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive",
                        "{error_msg}"
                    }
                }

                if *step.read() == 1 {
                    div { class: "space-y-3",
                        // P5 — pre-flight capability check so the operator
                        // sees whether their account already holds
                        // `cx.agent.provision` before submit. UI hint only;
                        // backend RBAC is canonical.
                        GrantedCapabilitiesView {
                            required_capability: Some("cx.agent.provision".to_string()),
                            title: Some("Required capability".to_string()),
                        }
                        Label { class: "text-sm".to_string(), "Controller DID" }
                        // P5 — ValidatedInput enforces the round-4
                        // did:<method>:<id> grammar inline so the operator
                        // sees the error immediately, not on submit.
                        ValidatedInput {
                            kind: ValidationKind::Did,
                            value: controller_did.read().clone(),
                            placeholder: "did:cx:abc123…".to_string(),
                            oninput: move |evt: FormEvent| controller_did.set(evt.value()),
                        }
                        Label { class: "text-sm".to_string(), "Display name (optional)" }
                        ValidatedInput {
                            kind: ValidationKind::MaxLength(128),
                            value: display_name.read().clone(),
                            oninput: move |evt: FormEvent| display_name.set(evt.value()),
                        }
                        p { class: "text-xs text-muted-foreground",
                            "Step 1/3 — DID request. The DID format MUST match the SDK normalize regex "
                            code { "^did:[a-z0-9]+:[^\\s]+$" } "."
                        }
                    }
                } else if *step.read() == 2 {
                    div { class: "space-y-3",
                        p { class: "text-sm",
                            "Step 2/3 — Authorize the first agent key. "
                            "TODO(P3-impl): runtime_attestation / self_asserted proof form."
                        }
                        p { class: "text-xs font-mono", "agent_principal_id = {agent_principal_id}" }
                    }
                } else {
                    div { class: "space-y-3",
                        p { class: "text-sm",
                            "Step 3/3 — Controller approval via coauth accountability_grant. "
                            "This issues a `cx:accountability_grant:<uuid7>` ledger row."
                        }
                    }
                }

                div { class: "flex justify-end gap-2 pt-2 border-t",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| close.call(()),
                        "Cancel"
                    }
                    if *step.read() == 1 {
                        Button {
                            onclick: move |_| {
                                let did = controller_did.read().clone();
                                let name = display_name.read().clone();
                                if !is_valid_did(&did) {
                                    error_msg.set("controller_did must match did:<method>:<id>".into());
                                    return;
                                }
                                error_msg.set(String::new());
                                spawn(async move {
                                    let req = AgentProvisionRequest {
                                        controller_did: did,
                                        display_name: if name.is_empty() { None } else { Some(name) },
                                        agent_key_proof: None,
                                    };
                                    match agents::provision_personal_agent(&req).await {
                                        Ok(resp) => {
                                            agent_principal_id.set(resp.agent_principal_id);
                                            step.set(2);
                                        }
                                        Err(e) => error_msg.set(e.message),
                                    }
                                });
                            },
                            "Continue"
                        }
                    } else if *step.read() == 2 {
                        Button {
                            onclick: move |_| step.set(3),
                            "Continue"
                        }
                    } else {
                        Button {
                            onclick: move |_| {
                                let id = agent_principal_id.read().clone();
                                let controller = controller_did.read().clone();
                                spawn(async move {
                                    let req = AccountabilityGrantRequest {
                                        controller_did: controller,
                                        rationale: Some("provisioned via sodmin wizard".into()),
                                    };
                                    match agents::issue_accountability_grant(&id, &req).await {
                                        Ok(_) => {
                                            show_toast("Personal agent provisioned", ToastVariant::Success);
                                            close.call(());
                                        }
                                        Err(e) => error_msg.set(e.message),
                                    }
                                });
                            },
                            "Finish"
                        }
                    }
                }
            }
        }
    }
}

/// DID format gate — delegates to the round-4 SDK normalize regex
/// (`^did:[a-z0-9]+:[^\s]+$`). Final write paths go through the
/// SDK's identifier parser which fails-closed on legacy forms.
fn is_valid_did(s: &str) -> bool {
    crate::utils::did::is_valid_did(s.trim())
}
