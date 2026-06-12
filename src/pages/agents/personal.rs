//! CKP-0008 / CKP-0009 personal-agent admin pages.
//!
//! Mounted at `/agents/personal` (list) and
//! `/agents/personal/:agent_id` (detail). Drives the 11 new soland
//! HTTP endpoints + the coauth `accountability_grant` mint via the
//! `ck.agent.manage` admin scope.
//!
//! TODO(P3-impl): soft validation of accountability-grant freshness
//! windows, deep validators on capability grant scopes, axe-core
//! assertions for the destructive deactivation flow.

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
use crate::types::{AccountabilityGrantRequest, Agent, AgentProvisionRequestBody};
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

const AGENT_CAPABILITY_ACTIONS: &[&str] = &[
    "ck.agent.key.authorize",
    "ck.agent.key.revoke",
    "ck.agent.key.rotate",
    "ck.self.agent.provision",
    "ck.self.agent.pause",
    "ck.self.agent.resume",
    "ck.self.agent.deactivate",
    "ck.agent.draft.propose",
    "ck.agent.action_request",
    "ck.agent.action_approve",
    "ck.agent.action_reject",
    "ck.self.agent.sidecar_thread.ensure",
    "ck.agent.sidecar_thread.write",
    "ck.agent.sidecar_thread.publish",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentConfirmAction {
    Pause,
    Resume,
    RotateKey,
    EnsureSidecar,
}

impl AgentConfirmAction {
    /// i18n key for the action label.
    fn label_key(self) -> &'static str {
        match self {
            Self::Pause => "agents.personal.pause",
            Self::Resume => "agents.personal.resume",
            Self::RotateKey => "agents.personal.rotate_key",
            Self::EnsureSidecar => "agents.personal.ensure_sidecar",
        }
    }

    fn phrase(self) -> &'static str {
        match self {
            Self::Pause => "PAUSE",
            Self::Resume => "RESUME",
            Self::RotateKey => "ROTATE",
            Self::EnsureSidecar => "ENSURE",
        }
    }

    /// i18n key for the confirmation-dialog description.
    fn description_key(self) -> &'static str {
        match self {
            Self::Pause => "agents.personal.confirm_pause_body",
            Self::Resume => "agents.personal.confirm_resume_body",
            Self::RotateKey => "agents.personal.confirm_rotate_body",
            Self::EnsureSidecar => "agents.personal.confirm_ensure_sidecar_body",
        }
    }
}

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
                title: t("agents.personal.title"),
                description: t("agents.personal.subtitle"),
                Button {
                    size: ButtonSize::Sm,
                    onclick: move |_| wizard_open.set(true),
                    {t("agents.personal.provision")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("agents.personal.agent_id")} }
                                    TableHead { {t("common.name")} }
                                    TableHead { {t("agents.personal.controller_did")} }
                                    TableHead { "actor_kind" }
                                    TableHead { {t("agents.personal.pairing")} }
                                    TableHead { {t("agents.personal.grant_freshness")} }
                                    TableHead { {t("common.status")} }
                                    TableHead { {t("common.actions")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { colspan: 99,
                                            class: "text-center text-muted-foreground py-8".to_string(),
                                            {t("agents.personal.no_agents")}
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
                    {t("common.open")}
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
    let mut pending_confirm_action = use_signal::<Option<AgentConfirmAction>>(|| None);
    let mut grant_action = use_signal(|| AGENT_CAPABILITY_ACTIONS[0].to_string());
    let mut grant_scope = use_signal(String::new);
    let mut grant_error = use_signal::<Option<String>>(|| None);

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: format!("{} {}", t("agents.personal.agent"), props.agent_id),
                description: t("agents.personal.detail_subtitle"),
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
                                h3 { class: "font-medium", {t("agents.personal.identity")} }
                                p { class: "text-sm", "actor_kind: " span { class: "font-mono", "{actor_kind}" } }
                                p { class: "text-sm", {t("agents.personal.pairing")} ": {pairing}" }
                                p { class: "text-sm", {t("agents.personal.grant_refreshed_at")} ": {freshness}" }
                            }
                            div { class: "rounded-md border p-4 space-y-2",
                                h3 { class: "font-medium", {t("agents.personal.agent_keys")} }
                                if agent_keys.is_empty() {
                                    p { class: "text-sm text-muted-foreground", {t("agents.personal.no_agent_keys")} }
                                } else {
                                    ul { class: "text-xs font-mono space-y-1",
                                        for key in agent_keys.iter() {
                                            li { "{key}" }
                                        }
                                    }
                                }
                            }
                            div { class: "rounded-md border p-4 space-y-2 md:col-span-2",
                                h3 { class: "font-medium", {t("agents.personal.capability_grants")} }
                                if capabilities.is_empty() {
                                    p { class: "text-sm text-muted-foreground", {t("agents.personal.no_active_grants")} }
                                } else {
                                    ul { class: "text-xs space-y-1",
                                        for cap in capabilities.iter() {
                                            li { class: "font-mono", "{cap}" }
                                        }
                                    }
                                }

                                div { class: "mt-4 grid gap-3 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto]",
                                    div { class: "space-y-1",
                                        Label { r#for: "agent-grant-action".to_string(), {t("agents.personal.grant_action")} }
                                        select {
                                            id: "agent-grant-action",
                                            class: "h-10 w-full rounded-md border bg-background px-3 py-2 text-sm",
                                            value: grant_action.read().clone(),
                                            oninput: move |evt: FormEvent| {
                                                grant_error.set(None);
                                                grant_action.set(evt.value());
                                            },
                                            for action in AGENT_CAPABILITY_ACTIONS.iter() {
                                                option {
                                                    value: *action,
                                                    selected: *action == grant_action.read().as_str(),
                                                    "{action}"
                                                }
                                            }
                                        }
                                    }
                                    div { class: "space-y-1",
                                        Label { r#for: "agent-grant-scope".to_string(), {t("agents.personal.scope_optional")} }
                                        ValidatedInput {
                                            kind: ValidationKind::MaxLength(256),
                                            value: grant_scope.read().clone(),
                                            placeholder: "ck:realm:... / ck:circle:...".to_string(),
                                            oninput: move |evt: FormEvent| {
                                                grant_error.set(None);
                                                grant_scope.set(evt.value());
                                            },
                                        }
                                    }
                                    div { class: "flex items-end",
                                        Button {
                                            size: ButtonSize::Sm,
                                            variant: ButtonVariant::Outline,
                                            onclick: move |_| {
                                                let id = agent_id_grant_attach.clone();
                                                let action = grant_action.read().trim().to_string();
                                                let scope = grant_scope.read().trim().to_string();
                                                if action.is_empty() {
                                                    grant_error.set(Some(t("agents.personal.select_action_first")));
                                                    return;
                                                }
                                                spawn(async move {
                                                    let scope_opt = if scope.is_empty() { None } else { Some(scope) };
                                                    match agents::attach_personal_agent_grant(
                                                        &id,
                                                        &action,
                                                        scope_opt.as_deref(),
                                                    ).await {
                                                        Ok(_) => show_toast(&t("agents.personal.toast_grant_attached"), ToastVariant::Success),
                                                        Err(e) => show_toast(&e.message, ToastVariant::Error),
                                                    }
                                                });
                                            },
                                            {t("agents.personal.attach_grant")}
                                        }
                                    }
                                }
                                if let Some(err) = grant_error.read().as_ref() {
                                    p { class: "text-xs text-destructive", "{err}" }
                                }
                            }
                        }

                        // Action buttons.
                        div { class: "flex flex-wrap gap-2 pt-4 border-t",
                            Button {
                                size: ButtonSize::Sm,
                                onclick: move |_| {
                                    let _ = agent_id_pause.clone();
                                    pending_confirm_action.set(Some(AgentConfirmAction::Pause));
                                },
                                {t("agents.personal.pause")}
                            }
                            Button {
                                size: ButtonSize::Sm,
                                variant: ButtonVariant::Secondary,
                                onclick: move |_| {
                                    let _ = agent_id_resume.clone();
                                    pending_confirm_action.set(Some(AgentConfirmAction::Resume));
                                },
                                {t("agents.personal.resume")}
                            }
                            Button {
                                size: ButtonSize::Sm,
                                variant: ButtonVariant::Outline,
                                onclick: move |_| {
                                    let _ = agent_id_rotate.clone();
                                    pending_confirm_action.set(Some(AgentConfirmAction::RotateKey));
                                },
                                {t("agents.personal.rotate_key")}
                            }
                            Button {
                                size: ButtonSize::Sm,
                                variant: ButtonVariant::Outline,
                                onclick: move |_| {
                                    let _ = agent_id_sidecar.clone();
                                    pending_confirm_action.set(Some(AgentConfirmAction::EnsureSidecar));
                                },
                                {t("agents.personal.ensure_sidecar")}
                            }
                            Button {
                                size: ButtonSize::Sm,
                                variant: ButtonVariant::Destructive,
                                onclick: move |_| deactivate_open.set(true),
                                {t("agents.personal.deactivate")}
                            }
                        }

                        if let Some(action) = pending_confirm_action() {
                            DangerousActionDialog {
                                open: true,
                                title: format!("{} {}", t(action.label_key()), t("agents.personal.personal_agent")),
                                description: t(action.description_key()),
                                confirmation_phrase: action.phrase().to_string(),
                                confirm_text: t(action.label_key()),
                                on_cancel: move |_| pending_confirm_action.set(None),
                                on_confirm: move |_| {
                                    let id = agent_id_for_actions.clone();
                                    pending_confirm_action.set(None);
                                    spawn(async move {
                                        match action {
                                            AgentConfirmAction::Pause => match agents::pause_personal_agent(&id).await {
                                                Ok(_) => show_toast(&t("agents.personal.toast_paused"), ToastVariant::Success),
                                                Err(e) => show_toast(&e.message, ToastVariant::Error),
                                            },
                                            AgentConfirmAction::Resume => match agents::resume_personal_agent(&id).await {
                                                Ok(_) => show_toast(&t("agents.personal.toast_resumed"), ToastVariant::Success),
                                                Err(e) => show_toast(&e.message, ToastVariant::Error),
                                            },
                                            AgentConfirmAction::RotateKey => match agents::rotate_personal_agent_key(&id).await {
                                                Ok(_) => show_toast(&t("agents.personal.toast_key_rotated"), ToastVariant::Success),
                                                Err(e) => show_toast(&e.message, ToastVariant::Error),
                                            },
                                            AgentConfirmAction::EnsureSidecar => match agents::ensure_sidecar_thread(&id).await {
                                                Ok(_) => show_toast(&t("agents.personal.toast_sidecar_ensured"), ToastVariant::Success),
                                                Err(e) => show_toast(&e.message, ToastVariant::Error),
                                            },
                                        }
                                    });
                                },
                            }
                        }

                        DangerousActionDialog {
                            open: *deactivate_open.read(),
                            title: t("agents.personal.deactivate_title"),
                            description: t("agents.personal.deactivate_body"),
                            confirmation_phrase: "DEACTIVATE".to_string(),
                            confirm_text: t("agents.personal.deactivate_confirm"),
                            on_cancel: move |_| deactivate_open.set(false),
                            on_confirm: move |_| {
                                let id = agent_id_deactivate.clone();
                                deactivate_open.set(false);
                                spawn(async move {
                                    match agents::deactivate_personal_agent(&id).await {
                                        Ok(_) => show_toast(&t("agents.personal.toast_deactivated"), ToastVariant::Success),
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
    // Pairing handshake returned by `ck.self.agent.provision`:
    // (pairing_request_id, pairing_code?, expires_at).
    let mut pairing_info = use_signal::<Option<(String, Option<String>, String)>>(|| None);
    let mut error_msg = use_signal(String::new);

    let close = props.on_close;

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
            div { class: "bg-background rounded-md border shadow-lg p-6 w-full max-w-lg space-y-4",
                h2 { class: "text-lg font-semibold", {format!("{} ({} {step}/3)", t("agents.personal.provision"), t("agents.personal.step"))} }

                if !error_msg.read().is_empty() {
                    div { class: "rounded border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive",
                        "{error_msg}"
                    }
                }

                if *step.read() == 1 {
                    div { class: "space-y-3",
                        // P5 — pre-flight capability check so the operator
                        // sees whether their account already holds
                        // `ck.self.agent.provision` before submit. UI hint only;
                        // backend RBAC is canonical.
                        GrantedCapabilitiesView {
                            required_capability: Some("ck.self.agent.provision".to_string()),
                            title: Some(t("agents.personal.required_capability")),
                        }
                        Label { class: "text-sm".to_string(), {t("agents.personal.controller_did")} }
                        // P5 — ValidatedInput enforces the round-4
                        // did:<method>:<id> grammar inline so the operator
                        // sees the error immediately, not on submit.
                        ValidatedInput {
                            kind: ValidationKind::Did,
                            value: controller_did.read().clone(),
                            placeholder: "did:web:alice.example".to_string(),
                            oninput: move |evt: FormEvent| controller_did.set(evt.value()),
                        }
                        Label { class: "text-sm".to_string(), {t("agents.personal.display_name")} }
                        ValidatedInput {
                            kind: ValidationKind::MaxLength(128),
                            value: display_name.read().clone(),
                            oninput: move |evt: FormEvent| display_name.set(evt.value()),
                        }
                        p { class: "text-xs text-muted-foreground",
                            {t("agents.personal.wizard_step1")} " "
                            code { "^did:[a-z0-9]+:[^\\s]+$" } "."
                        }
                    }
                } else if *step.read() == 2 {
                    div { class: "space-y-3",
                        p { class: "text-sm",
                            {t("agents.personal.wizard_step2")}
                        }
                        p { class: "text-xs text-muted-foreground",
                            {t("agents.personal.wizard_step2_hint")}
                        }
                    }
                } else {
                    div { class: "space-y-3",
                        p { class: "text-sm",
                            {t("agents.personal.wizard_step3")}
                        }
                        if let Some((request_id, code, expires_at)) = pairing_info.read().clone() {
                            div { class: "rounded-md border bg-muted/30 p-3 space-y-1 text-xs font-mono",
                                div { "pairing_request_id: {request_id}" }
                                if let Some(code) = code {
                                    div { "pairing_code: {code}" }
                                }
                                div { "expires_at: {expires_at}" }
                            }
                            p { class: "text-xs text-muted-foreground",
                                {t("agents.personal.relay_pairing_code")}
                            }
                        }
                    }
                }

                div { class: "flex justify-end gap-2 pt-2 border-t",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| close.call(()),
                        {t("common.cancel")}
                    }
                    if *step.read() == 1 {
                        Button {
                            onclick: move |_| {
                                let did = controller_did.read().clone();
                                if !is_valid_did(&did) {
                                    error_msg.set(t("agents.personal.invalid_controller_did"));
                                    return;
                                }
                                error_msg.set(String::new());
                                step.set(2);
                            },
                            {t("agents.personal.continue")}
                        }
                    } else if *step.read() == 2 {
                        Button {
                            onclick: move |_| {
                                let name = display_name.read().clone();
                                error_msg.set(String::new());
                                spawn(async move {
                                    // Spec `agent_provision_request_body`
                                    // — the controller is the
                                    // authenticated principal; no
                                    // controller_did / key-proof fields.
                                    let req = AgentProvisionRequestBody {
                                        display_name: if name.is_empty() { None } else { Some(name) },
                                        agent_slug: None,
                                        requested_scope: None,
                                        accountability: serde_json::Value::Null,
                                        pairing_ttl_ms: None,
                                    };
                                    match agents::provision_personal_agent(&req).await {
                                        Ok(resp) => {
                                            agent_principal_id.set(resp.agent_principal_id.to_string());
                                            pairing_info.set(Some((
                                                resp.pairing_request_id.clone(),
                                                resp.pairing_code.clone(),
                                                resp.expires_at.to_rfc3339(),
                                            )));
                                            step.set(3);
                                        }
                                        Err(e) => error_msg.set(e.message),
                                    }
                                });
                            },
                            {t("agents.personal.provision_agent")}
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
                                            show_toast(&t("agents.personal.toast_provisioned"), ToastVariant::Success);
                                            close.call(());
                                        }
                                        Err(e) => error_msg.set(e.message),
                                    }
                                });
                            },
                            {t("agents.personal.finish")}
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
    crate::utils::security::did::is_valid_did(s.trim())
}
