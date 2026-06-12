//! Notary cell admin page (Stream H', H'2).
//!
//! Renders the current `ck:cell:ck.component.notary.v1:<realm_id>` cell
//! value (single_did / threshold / open_set / mixed) and exposes a form
//! that constructs a notary reconfig Move. The submit path posts the
//! SDK-authoritative `NotaryValue` to soland's
//! `/_soland/admin/realms/{realm_id}/notary/reconfigure` endpoint;
//! soland builds the typed Move + signs with the admin's signer flow.
//!
//! Spec rule: a new notary cannot self-sign itself in. We mirror that
//! constraint client-side via `types::seal::self_sign_violation` so the
//! operator gets a hard pre-flight stop before paying a round-trip.

use cokret_core::Did;
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
    NotaryValue, NotaryValueExt, SubmitMoveOutcome, admin_self_signs_themselves_in,
};
use crate::utils::net::session;

/// Build the SDK-authoritative [`NotaryValue`] from the raw form state.
/// Returns a human-readable error when a field is missing / not a valid
/// DID / structurally inconsistent (the same rules
/// `NotaryValue::validate()` enforces server-side).
fn notary_value_from_form(
    kind: &str,
    single_did: &str,
    threshold_k: &str,
    threshold_n: &str,
    threshold_members: &str,
    open_set_members: &str,
    mixed_primary: &str,
    mixed_recovery: &str,
) -> Result<NotaryValue, String> {
    let parse_did = |raw: &str, field: &str| -> Result<Did, String> {
        Did::new(raw.trim().to_owned()).map_err(|e| format!("{field}: invalid DID ({e})"))
    };
    let parse_did_lines = |raw: &str, field: &str| -> Result<Vec<Did>, String> {
        split_lines(raw)
            .iter()
            .map(|line| parse_did(line, field))
            .collect()
    };
    let value = match kind {
        "single_did" => NotaryValue::SingleDid {
            did: parse_did(single_did, "DID")?,
        },
        "threshold" => NotaryValue::Threshold {
            k: parse_u32(threshold_k).ok_or("k: must be a positive integer")?,
            n: parse_u32(threshold_n).ok_or("n: must be a positive integer")?,
            members: parse_did_lines(threshold_members, "members")?,
        },
        "open_set" => NotaryValue::OpenSet {
            members: parse_did_lines(open_set_members, "members")?,
        },
        "mixed" => NotaryValue::Mixed {
            primary: parse_did(mixed_primary, "primary")?,
            recovery_members: parse_did_lines(mixed_recovery, "recovery members")?,
        },
        other => return Err(format!("unknown notary kind: {other}")),
    };
    value.validate().map_err(|e| e.to_string())?;
    Ok(value)
}

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
    // Pending reconfig value the user has clicked "Construct Move" on
    // but not yet confirmed. While `Some`, the confirmation modal is
    // visible. We carry the typed value rather than re-reading the
    // form signals so the body the user confirmed is what we POST.
    let mut pending = use_signal::<Option<NotaryValue>>(|| None);
    // H'2 round 27 — last successful soland response. Drives the
    // readonly outcome viewer below the form so the admin can verify
    // the submitted Move id / Seal id before walking away.
    let mut last_response = use_signal::<Option<SubmitMoveOutcome>>(|| None);

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
                Some(Ok(cell)) => rsx! {
                    Card {
                        CardHeader { CardTitle { "Current notary value" } }
                        CardContent {
                            div { class: "space-y-2 text-sm",
                                div { class: "flex items-center gap-2",
                                    Badge { variant: BadgeVariant::Secondary, "{cell.value.kind_label()}" }
                                    span { class: "text-muted-foreground", "{cell.value.summary()}" }
                                    if cell.paused {
                                        Badge { variant: BadgeVariant::Destructive, "paused" }
                                    }
                                }
                                {render_value_detail(&cell.value)}
                                if let Some(ms) = cell.revocation_freshness_window_ms {
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
                CardHeader { CardTitle { "Construct notary reconfig Move" } }
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
                                Label { r#for: "notary-threshold-members".to_string(),
                                    "Members (one DID per line)"
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
                                    let value = match notary_value_from_form(
                                        &new_kind.read(),
                                        &new_single_did.read(),
                                        &new_threshold_k.read(),
                                        &new_threshold_n.read(),
                                        &new_threshold_dids.read(),
                                        &new_open_set_members.read(),
                                        &new_mixed_primary.read(),
                                        &new_mixed_recovery.read(),
                                    ) {
                                        Ok(v) => v,
                                        Err(msg) => {
                                            show_toast(&msg, ToastVariant::Error);
                                            return;
                                        }
                                    };
                                    // Spec rule: "new notary cannot
                                    // self-sign itself in". soland will
                                    // reject this server-side too, but
                                    // we hard-block client-side so the
                                    // operator can correct the form
                                    // before paying a round-trip.
                                    if let Some(did) = &admin_did_for_submit
                                        && admin_self_signs_themselves_in(&value, did) {
                                            show_toast(
                                                "Refusing to submit: the proposed notary includes the current admin DID. Pick a different operator (or, if you intend the swap, perform it via a fresh admin scope, not self-signed).",
                                                ToastVariant::Error,
                                            );
                                            return;
                                        }
                                    pending.set(Some(value));
                                },
                                "Construct Move"
                            }
                        }
                    }
                }
            }

            // H'2 round 27 — readonly outcome viewer. Surfaces the Move id
            // (and the Seal id when the in-process notary worker already
            // folded the Move) so the operator can verify the submission
            // before walking away from the page.
            {
                let snapshot = last_response.read().clone();
                if let Some(resp) = snapshot {
                    let move_id = resp.move_id.clone();
                    let status = resp.status.clone();
                    let seal_id = resp.seal_id.clone();
                    let reason = resp.reason.clone();
                    rsx! {
                        Card {
                            CardHeader { CardTitle { "Last submitted Move" } }
                            CardContent {
                                div { class: "space-y-2 text-sm",
                                    div { class: "flex items-center gap-2",
                                        Badge { variant: BadgeVariant::Secondary, "move_id" }
                                        span { class: "font-mono text-xs break-all", "{move_id}" }
                                    }
                                    div { class: "flex items-center gap-2",
                                        Badge { variant: BadgeVariant::Secondary, "status" }
                                        span { class: "font-mono text-xs", "{status}" }
                                    }
                                    if let Some(seal_id) = seal_id {
                                        div { class: "flex items-center gap-2",
                                            Badge { variant: BadgeVariant::Secondary, "seal_id" }
                                            span { class: "font-mono text-xs break-all", "{seal_id}" }
                                        }
                                    }
                                    if let Some(reason) = reason {
                                        p { class: "text-xs text-muted-foreground", "{reason}" }
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
            // value is staged. Highlights that this is a high-sensitivity
            // admin operation and re-states the proposed kind so the
            // operator has one last chance to back out.
            {
                let pending_snapshot = pending.read().clone();
                let open = pending_snapshot.is_some();
                let title = "Submit notary reconfig?".to_string();
                let admin_did_for_warn = admin_did_for_modal.clone();
                let description = match &pending_snapshot {
                    Some(value) => {
                        let mut d = format!(
                            "High-sensitivity admin operation. Proposed kind = {}. Confirm your admin scope is current; soland will reject if scope has lapsed.",
                            value.kind_label()
                        );
                        if let Some(did) = admin_did_for_warn.as_deref()
                            && admin_self_signs_themselves_in(value, did) {
                                d.push_str(" Warning: the proposed notary references the current admin DID; this will be rejected.");
                            }
                        d
                    }
                    None => String::new(),
                };
                let confirm_text = if *submitting.read() {
                    "Submitting…".to_string()
                } else {
                    "Submit Move".to_string()
                };
                let realm_id_for_confirm = realm_id_for_submit.clone();
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
                            if let Some(value) = staged {
                                submitting.set(true);
                                let realm_id = realm_id_for_confirm.clone();
                                spawn(async move {
                                    match seal::submit_notary_reconfig(&realm_id, &value).await {
                                        Ok(resp) => {
                                            show_toast(
                                                &format!("Move submitted: {}", resp.move_id),
                                                ToastVariant::Success,
                                            );
                                            // H'2 — surface the outcome in
                                            // the readonly viewer panel
                                            // below.
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
    match v {
        NotaryValue::SingleDid { did } => {
            let did = did.to_string();
            rsx! {
                div { class: "font-mono text-xs", "did: {did}" }
            }
        }
        NotaryValue::Threshold { k, n, members } => {
            let k = *k;
            let n = *n;
            let members: Vec<String> = members.iter().map(|d| d.to_string()).collect();
            rsx! {
                div { class: "font-mono text-xs", "k/n: {k}/{n}" }
                ul { class: "list-disc list-inside text-xs font-mono",
                    for d in members.iter() { li { "{d}" } }
                }
            }
        }
        NotaryValue::OpenSet { members } => {
            let members: Vec<String> = members.iter().map(|d| d.to_string()).collect();
            rsx! {
                ul { class: "list-disc list-inside text-xs font-mono",
                    for m in members.iter() { li { "{m}" } }
                }
            }
        }
        NotaryValue::Mixed {
            primary,
            recovery_members,
        } => {
            let primary = primary.to_string();
            let recovery: Vec<String> = recovery_members.iter().map(|d| d.to_string()).collect();
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

pub(crate) fn parse_u32(raw: &str) -> Option<u32> {
    raw.trim().parse::<u32>().ok()
}

#[cfg(test)]
mod tests {
    use super::{notary_value_from_form, parse_u32, split_lines};
    use crate::types::seal::{NotaryValue, NotaryValueExt};

    #[test]
    fn split_lines_trims_and_drops_empty() {
        let raw = "did:a\n  did:b  \n\n\ndid:c\n";
        let v = split_lines(raw);
        assert_eq!(v, vec!["did:a".to_string(), "did:b".into(), "did:c".into()]);
    }

    #[test]
    fn parse_u32_handles_invalid() {
        assert_eq!(parse_u32(" 7 "), Some(7));
        assert!(parse_u32("").is_none());
        assert!(parse_u32("not-a-number").is_none());
    }

    #[test]
    fn form_builds_validated_sdk_value() {
        let value = notary_value_from_form(
            "threshold",
            "",
            "2",
            "3",
            "did:ck:a\ndid:ck:b\ndid:ck:c",
            "",
            "",
            "",
        )
        .expect("valid threshold form");
        assert_eq!(value.kind_label(), "threshold");
        match value {
            NotaryValue::Threshold { k, n, members } => {
                assert_eq!((k, n), (2, 3));
                assert_eq!(members.len(), 3);
            }
            other => panic!("expected Threshold, got {other:?}"),
        }

        // Structural violations are caught client-side by the same SDK
        // validator soland runs server-side (members.len() != n here).
        assert!(
            notary_value_from_form("threshold", "", "2", "3", "did:ck:a", "", "", "").is_err()
        );
        // Invalid DID is rejected before constructing the value.
        assert!(
            notary_value_from_form("single_did", "not-a-did", "", "", "", "", "", "").is_err()
        );
    }
}
