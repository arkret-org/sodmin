//! Notary cell admin page (Stream H', H'2).
//!
//! Renders the current `ck:cell:ck.component.notary.v1:<realm_id>` cell
//! value (single_did / threshold / open_set / mixed) and exposes a form
//! that constructs a notary reconfig Control Move. The submit path posts to
//! soland's `/_soland/admin/realms/{realm_id}/notary/reconfigure` endpoint;
//! soland builds the typed Control Move + signs with the admin's signer strand.
//!
//! Spec rule: a new notary cannot self-sign itself in. We mirror that
//! constraint client-side via `NotaryReconfigRequest::admin_self_signs_themselves_in`
//! so the operator gets a hard pre-flight stop before paying a round-trip.

use dioxus::prelude::*;

use crate::api::seal;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::seal::{
    NotaryKind, NotaryReconfigRequest, NotaryValue, SubmitControlMoveOutcome,
};
use crate::utils::net::session;

#[component]
pub fn NotaryPage(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let realm_id_for_fetch = realm_id.clone();
    let mut data = use_resource(move || {
        let id = realm_id_for_fetch.clone();
        async move { seal::get_notary_value(&id).await }
    });

    let mut new_kind = use_signal(|| "single_did".to_string());
    let mut new_single_did = use_signal(String::new);
    let mut new_threshold_k = use_signal(|| "2".to_string());
    let mut new_threshold_n = use_signal(|| "3".to_string());
    let mut new_threshold_dids = use_signal(String::new);
    let mut new_open_set_members = use_signal(String::new);
    let mut new_mixed_primary = use_signal(String::new);
    let mut new_mixed_recovery = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    // Pending reconfig request the user has clicked "Construct Control Move" on
    // but not yet confirmed. While `Some`, the confirmation modal is
    // visible. We carry the typed request rather than re-reading the
    // form signals so the body the user confirmed is what we POST.
    let mut pending = use_signal::<Option<NotaryReconfigRequest>>(|| None);
    // H'2 round 27 — last successful soland response. Drives the
    // "Signed Control Move body" readonly JSON viewer below the form so the
    // admin can verify byte-for-byte what was signed before walking
    // away from the page.
    let mut last_response = use_signal::<Option<SubmitControlMoveOutcome>>(|| None);

    // Best-effort admin DID. Used both for the spec-rule pre-check
    // ("new notary cannot self-sign itself in") and for the warning
    // banner that prompts the operator to re-check their admin scope
    // before submitting. `user_id` is what the OAuth callback persists;
    // for did:key admins this is the did string itself, for opaque
    // admins it's a stable id we still treat as the DID for comparison.
    let admin_did: Option<String> = session::current_user().id.filter(|s| !s.is_empty());
    let admin_did_for_submit = admin_did.clone();
    let admin_did_for_modal = admin_did.clone();

    let realm_id_for_submit = realm_id.clone();
    let header_realm_id = realm_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: format!("Notary · {}", header_realm_id),
                description: "Configure the notary cell for this Realm (single_did / threshold / open_set / mixed).".to_string(),
            }

            match &*data.read() {
                Some(Ok(value)) => rsx! {
                    Card {
                        CardHeader { CardTitle { "Current notary value" } }
                        CardContent {
                            div { class: "space-y-2 text-sm",
                                div { class: "flex items-center gap-2",
                                    Badge { variant: BadgeVariant::Secondary, "{value.kind_label()}" }
                                    span { class: "text-muted-foreground", "{value.summary()}" }
                                    if value.paused {
                                        Badge { variant: BadgeVariant::Destructive, "paused" }
                                    }
                                }
                                {render_value_detail(value)}
                                if let Some(ms) = value.revocation_freshness_window_ms {
                                    p { class: "text-muted-foreground",
                                        {format!("revocation_freshness_window_ms: {}", ms)}
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

            Card {
                CardHeader { CardTitle { "Construct notary reconfig Control Move" } }
                CardContent {
                    div { class: "space-y-3",
                        div { class: "space-y-1",
                            Label { r#for: "notary-kind".to_string(), "Kind" }
                            select {
                                class: "flex h-10 w-full rounded-md border bg-background px-3 py-2 text-sm",
                                value: new_kind.read().clone(),
                                onchange: move |evt: FormEvent| new_kind.set(evt.value()),
                                option { value: "single_did", "single_did" }
                                option { value: "threshold", "threshold" }
                                option { value: "open_set", "open_set" }
                                option { value: "mixed", "mixed" }
                            }
                        }

                        if *new_kind.read() == "single_did" {
                            div { class: "space-y-1",
                                Label { r#for: "notary-single-did".to_string(), "DID" }
                                Input {
                                    value: new_single_did.read().clone(),
                                    oninput: move |evt: FormEvent| new_single_did.set(evt.value()),
                                }
                            }
                        }

                        if *new_kind.read() == "threshold" {
                            div { class: "grid grid-cols-2 gap-3",
                                div { class: "space-y-1",
                                    Label { r#for: "notary-k".to_string(), "k" }
                                    Input {
                                        r#type: "number".to_string(),
                                        value: new_threshold_k.read().clone(),
                                        oninput: move |evt: FormEvent| new_threshold_k.set(evt.value()),
                                    }
                                }
                                div { class: "space-y-1",
                                    Label { r#for: "notary-n".to_string(), "n" }
                                    Input {
                                        r#type: "number".to_string(),
                                        value: new_threshold_n.read().clone(),
                                        oninput: move |evt: FormEvent| new_threshold_n.set(evt.value()),
                                    }
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "notary-threshold-dids".to_string(),
                                    "DIDs (one per line)"
                                }
                                textarea {
                                    class: "flex min-h-[80px] w-full rounded-md border bg-background px-3 py-2 text-sm",
                                    value: new_threshold_dids.read().clone(),
                                    oninput: move |evt: FormEvent| new_threshold_dids.set(evt.value()),
                                }
                            }
                        }

                        if *new_kind.read() == "open_set" {
                            div { class: "space-y-1",
                                Label { r#for: "notary-open-set".to_string(),
                                    "Members (one DID per line)"
                                }
                                textarea {
                                    class: "flex min-h-[80px] w-full rounded-md border bg-background px-3 py-2 text-sm",
                                    value: new_open_set_members.read().clone(),
                                    oninput: move |evt: FormEvent| new_open_set_members.set(evt.value()),
                                }
                            }
                        }

                        if *new_kind.read() == "mixed" {
                            div { class: "space-y-1",
                                Label { r#for: "notary-mixed-primary".to_string(), "Primary DID" }
                                Input {
                                    value: new_mixed_primary.read().clone(),
                                    oninput: move |evt: FormEvent| new_mixed_primary.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "notary-mixed-recovery".to_string(),
                                    "Recovery DIDs (one per line)"
                                }
                                textarea {
                                    class: "flex min-h-[80px] w-full rounded-md border bg-background px-3 py-2 text-sm",
                                    value: new_mixed_recovery.read().clone(),
                                    oninput: move |evt: FormEvent| new_mixed_recovery.set(evt.value()),
                                }
                            }
                        }

                        div { class: "flex justify-end",
                            Button {
                                variant: ButtonVariant::Default,
                                disabled: *submitting.read(),
                                onclick: move |_| {
                                    let kind = new_kind.read().clone();
                                    let req = NotaryReconfigRequest {
                                        realm_id: realm_id_for_submit.clone(),
                                        kind: kind.clone(),
                                        single_did: opt_string(&new_single_did.read()),
                                        threshold_k: parse_u32(&new_threshold_k.read()),
                                        threshold_n: parse_u32(&new_threshold_n.read()),
                                        threshold_dids: split_lines(&new_threshold_dids.read()),
                                        open_set_members: split_lines(&new_open_set_members.read()),
                                        mixed_primary: opt_string(&new_mixed_primary.read()),
                                        mixed_recovery: split_lines(&new_mixed_recovery.read()),
                                    };
                                    // Spec rule: "new notary cannot
                                    // self-sign itself in". soland will
                                    // reject this server-side too, but
                                    // we hard-block client-side so the
                                    // operator can correct the form
                                    // before paying a round-trip.
                                    if let Some(did) = &admin_did_for_submit
                                        && req.admin_self_signs_themselves_in(did) {
                                            show_toast(
                                                "Refusing to submit: the proposed notary includes the current admin DID. Pick a different operator (or, if you intend the swap, perform it via a fresh admin scope, not self-signed).",
                                                ToastVariant::Error,
                                            );
                                            return;
                                        }
                                    pending.set(Some(req));
                                },
                                "Construct Control Move"
                            }
                        }
                    }
                }
            }

            // H'2 round 27 — readonly Control Move body viewer. Surfaces the
            // canonical signed body that soland built on the admin's
            // behalf so the operator can verify byte-for-byte what was
            // signed. Collapsed by default via native `<details>`; no
            // JS, no copy-paste affordance (admins can use browser
            // built-ins).
            {
                let snapshot = last_response.read().clone();
                if let Some(resp) = snapshot {
                    let control_move_id = resp.control_move_id.clone();
                    let pretty = serde_json::to_string_pretty(&resp.control_move_body)
                        .unwrap_or_else(|_| "(failed to render control_move_body)".to_string());
                    rsx! {
                        Card {
                            CardHeader { CardTitle { "Last submitted Control Move body" } }
                            CardContent {
                                div { class: "space-y-2 text-sm",
                                    div { class: "flex items-center gap-2",
                                        Badge { variant: BadgeVariant::Secondary, "control_move_id" }
                                        span { class: "font-mono text-xs break-all", "{control_move_id}" }
                                    }
                                    details { class: "rounded-md border bg-muted/30",
                                        summary { class: "cursor-pointer select-none px-3 py-2 text-xs font-medium",
                                            "Signed control_move_body"
                                        }
                                        pre { class: "px-3 py-2 text-xs font-mono whitespace-pre-wrap break-all",
                                            "{pretty}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            }

            // Confirmation modal. Shown whenever a pending reconfig
            // request is staged. Highlights that this is a high-sensitivity
            // admin operation and re-states the proposed kind so the
            // operator has one last chance to back out.
            {
                let pending_snapshot = pending.read().clone();
                let open = pending_snapshot.is_some();
                let title = "Submit notary reconfig?".to_string();
                let admin_did_for_warn = admin_did_for_modal.clone();
                let description = match &pending_snapshot {
                    Some(req) => {
                        let mut d = format!(
                            "High-sensitivity admin operation. Proposed kind = {}. Confirm your admin scope is current; soland will reject if scope has lapsed.",
                            req.kind
                        );
                        if let Some(did) = admin_did_for_warn.as_deref()
                            && req.admin_self_signs_themselves_in(did) {
                                d.push_str(" Warning: the proposed notary references the current admin DID; this will be rejected.");
                            }
                        d
                    }
                    None => String::new(),
                };
                let confirm_text = if *submitting.read() {
                    "Submitting…".to_string()
                } else {
                    "Submit Control Move".to_string()
                };
                rsx! {
                    ConfirmDialog {
                        open,
                        title,
                        description,
                        confirm_text,
                        cancel_text: "Cancel".to_string(),
                        destructive: true,
                        on_cancel: move |_| pending.set(None),
                        on_confirm: move |_| {
                            if *submitting.read() { return; }
                            let staged = pending.read().clone();
                            if let Some(req) = staged {
                                submitting.set(true);
                                spawn(async move {
                                    match seal::submit_notary_reconfig(&req).await {
                                        Ok(resp) => {
                                            show_toast(
                                                &format!("Control Move submitted: {}", resp.control_move_id),
                                                ToastVariant::Success,
                                            );
                                            // H'2 — surface the signed
                                            // Control Move body in the readonly
                                            // JSON viewer panel below.
                                            last_response.set(Some(resp));
                                        }
                                        Err(e) => show_toast(
                                            &format!("Failed: {}", e.message),
                                            ToastVariant::Error,
                                        ),
                                    }
                                    submitting.set(false);
                                    pending.set(None);
                                });
                            }
                        },
                    }
                }
            }
        }
    }
}

fn render_value_detail(v: &NotaryValue) -> Element {
    match v.kind() {
        None => {
            let raw = v.kind_raw.clone();
            rsx! {
                div { class: "font-mono text-xs text-muted-foreground", "unknown notary kind: {raw}" }
            }
        }
        Some(NotaryKind::SingleDid) => {
            let did = v.single_did.clone().unwrap_or_else(|| "-".to_string());
            rsx! {
                div { class: "font-mono text-xs", "did: {did}" }
            }
        }
        Some(NotaryKind::Threshold) => {
            let k = v.threshold_k.unwrap_or(0);
            let n = v.threshold_n.unwrap_or(0);
            let dids = v.threshold_dids.clone();
            rsx! {
                div { class: "font-mono text-xs", "k/n: {k}/{n}" }
                ul { class: "list-disc list-inside text-xs font-mono",
                    for d in dids.iter() { li { "{d}" } }
                }
            }
        }
        Some(NotaryKind::OpenSet) => {
            let members = v.open_set_members.clone();
            rsx! {
                ul { class: "list-disc list-inside text-xs font-mono",
                    for m in members.iter() { li { "{m}" } }
                }
            }
        }
        Some(NotaryKind::Mixed) => {
            let primary = v.mixed_primary.clone().unwrap_or_else(|| "-".to_string());
            let recovery = v.mixed_recovery.clone();
            rsx! {
                div { class: "font-mono text-xs", "primary: {primary}" }
                div { class: "text-xs text-muted-foreground", "recovery:" }
                ul { class: "list-disc list-inside text-xs font-mono",
                    for r in recovery.iter() { li { "{r}" } }
                }
            }
        }
    }
}

pub(crate) fn split_lines(raw: &str) -> Vec<String> {
    raw.lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub(crate) fn opt_string(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn parse_u32(raw: &str) -> Option<u32> {
    raw.trim().parse::<u32>().ok()
}

#[cfg(test)]
mod tests {
    use super::{opt_string, parse_u32, split_lines};

    #[test]
    fn split_lines_trims_and_drops_empty() {
        let raw = "did:a\n  did:b  \n\n\ndid:c\n";
        let v = split_lines(raw);
        assert_eq!(v, vec!["did:a".to_string(), "did:b".into(), "did:c".into()]);
    }

    #[test]
    fn opt_string_empty_becomes_none() {
        assert!(opt_string("").is_none());
        assert!(opt_string("   ").is_none());
        assert_eq!(opt_string("  hi  "), Some("hi".to_string()));
    }

    #[test]
    fn parse_u32_handles_invalid() {
        assert_eq!(parse_u32(" 7 "), Some(7));
        assert!(parse_u32("").is_none());
        assert!(parse_u32("not-a-number").is_none());
    }
}
