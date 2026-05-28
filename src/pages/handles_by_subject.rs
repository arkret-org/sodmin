//! R3.2 (UI-SOD-4) — "Subject → Handles" directory admin page.
//!
//! Route: `/admin/handles/by-subject?subject=did:...`
//!
//! Given a known holder/principal DID the operator can look up the
//! currently visible signed `cx.schema.handle_claim.v1` evidence via
//! `cx.directory.list_handles_for_subject` (the inverse of
//! `resolve_handle`). The directory applies disclosure policy / issuer
//! trust / audience / Realm-intent filtering server-side; this page
//! renders the visible claims plus the §3.2.1 primary handle.
//!
//! Each claim row carries a "Why am I seeing this?" tooltip exposing the
//! issuer DID + binding_state + created_at so the operator understands
//! the disclosure provenance.
//!
//! `MemberIdentity` no longer carries handle fields (contrix-spec @
//! b56cab1) — this page is the operator-facing way to inspect a
//! subject's handle bindings, replacing the old MemberIdentity drilldown.
//!
//! Data plumbing is minimal: the page issues the directory call and
//! re-derives the primary handle locally via the SDK-mirror
//! [`crate::utils::primary_handle::select_primary_handle`].

use chrono::Utc;
use dioxus::prelude::*;

use crate::api::directory;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::Button;
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::types::{HandleBindingState, HandleClaim, ListHandlesForSubjectRequest};
use crate::utils::did;
use crate::utils::handle::display_sigil;
use crate::utils::i18n::t;
use crate::utils::primary_handle::{PrimaryHandleSelectInput, select_primary_handle_string};

#[component]
pub fn HandlesBySubject(subject: Option<String>) -> Element {
    let initial = subject.unwrap_or_default();
    let mut input = use_signal(|| initial.clone());
    // The submitted query that drives the directory call. Empty = no
    // lookup issued yet (render the "enter a DID" empty state).
    let mut query = use_signal(|| initial.clone());

    let query_val = query.read().clone();
    let mut data = use_resource(move || {
        let subject = query_val.clone();
        async move {
            if subject.trim().is_empty() {
                return None;
            }
            let req = ListHandlesForSubjectRequest {
                subject: subject.trim().to_string(),
                intent: Some("admin_directory".to_string()),
                ..Default::default()
            };
            Some(directory::list_handles_for_subject(&req).await)
        }
    });

    let input_val = input.read().clone();
    let trimmed = input_val.trim().to_string();
    let can_submit = did::is_valid_did(&trimmed);

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("handles_by_subject.title"),
                description: t("handles_by_subject.subtitle"),
            }

            Card {
                CardHeader {
                    CardTitle { class: "text-lg".to_string(), {t("handles_by_subject.lookup_title")} }
                    CardDescription { {t("handles_by_subject.lookup_body")} }
                }
                CardContent {
                    form {
                        class: "flex flex-col gap-2 sm:flex-row sm:items-end",
                        onsubmit: move |evt| {
                            evt.prevent_default();
                            let v = input.read().trim().to_string();
                            if did::is_valid_did(&v) {
                                query.set(v);
                            }
                        },
                        div { class: "flex-1 space-y-1",
                            Label { r#for: "subject-did".to_string(), {t("handles_by_subject.subject_did")} }
                            Input {
                                id: "subject-did".to_string(),
                                r#type: "text".to_string(),
                                placeholder: "did:web:alice.example".to_string(),
                                value: input.read().clone(),
                                oninput: move |evt: FormEvent| input.set(evt.value()),
                            }
                        }
                        Button {
                            r#type: "submit".to_string(),
                            disabled: !can_submit,
                            {t("handles_by_subject.lookup")}
                        }
                    }
                }
            }

            match &*data.read() {
                None => rsx! { PageSkeleton {} },
                Some(None) => rsx! {
                    div { class: "rounded-md border border-dashed px-4 py-8 text-center text-sm text-muted-foreground",
                        {t("handles_by_subject.enter_did")}
                    }
                },
                Some(Some(Err(e))) => rsx! {
                    ErrorBanner { message: e.message.clone(), on_retry: move |_| data.restart() }
                },
                Some(Some(Ok(resp))) => {
                    let subject_id = resp.subject.clone();
                    // Defensive fail-closed view: drop claims whose
                    // subject != response.subject (schema invariant).
                    let visible: Vec<HandleClaim> =
                        resp.visible_claims().into_iter().cloned().collect();
                    // Re-derive the §3.2.1 primary handle locally; fall
                    // back to the server-reported value.
                    let derived_primary = {
                        let sel = PrimaryHandleSelectInput {
                            subject_id: &subject_id,
                            context: None,
                            claim_set_snapshot: &visible,
                            accepted_issuers: &[],
                            holder_primary_handle_at_as_of: None,
                            resolution_as_of: Utc::now(),
                        };
                        select_primary_handle_string(&sel)
                    };
                    let primary = derived_primary.or_else(|| resp.primary_handle.clone());
                    rsx! {
                        {results_card(subject_id.clone(), primary, visible, resp.has_more)}
                    }
                }
            }
        }
    }
}

fn results_card(
    subject_id: String,
    primary: Option<String>,
    claims: Vec<HandleClaim>,
    has_more: bool,
) -> Element {
    let primary_label = primary.clone().unwrap_or_else(|| "-".to_string());
    let primary_sigil = primary.as_deref().map(display_sigil).unwrap_or_default();
    rsx! {
        Card {
            CardHeader {
                div { class: "flex flex-wrap items-start justify-between gap-2",
                    div { class: "space-y-1",
                        CardTitle { class: "text-lg".to_string(), {t("handles_by_subject.results_title")} }
                        CardDescription { class: "font-mono break-all".to_string(), "{subject_id}" }
                    }
                    div { class: "flex flex-col items-end",
                        span { class: "text-xs text-muted-foreground", {t("handles_by_subject.primary_handle")} }
                        span { class: "text-sm font-semibold font-mono", "{primary_label}" }
                        if !primary_sigil.is_empty() {
                            span { class: "text-muted-foreground/80 text-[10px]", "{primary_sigil}" }
                        }
                    }
                }
            }
            CardContent {
                Table {
                    TableHeader {
                        TableRow {
                            TableHead { {t("handles_by_subject.col_handle")} }
                            TableHead { {t("handles_by_subject.col_issuer")} }
                            TableHead { {t("handles_by_subject.col_binding")} }
                            TableHead { {t("handles_by_subject.col_expires")} }
                            TableHead { {t("handles_by_subject.col_why")} }
                        }
                    }
                    TableBody {
                        if claims.is_empty() {
                            TableRow {
                                TableCell {
                                    class: "text-center text-muted-foreground py-6".to_string(),
                                    colspan: 99,
                                    {t("handles_by_subject.no_claims")}
                                }
                            }
                        } else {
                            for claim in claims.iter() {
                                {claim_row(claim, primary.as_deref())}
                            }
                        }
                    }
                }
                if has_more {
                    p { class: "mt-3 text-xs text-muted-foreground",
                        {t("handles_by_subject.has_more")}
                    }
                }
            }
        }
    }
}

fn claim_row(claim: &HandleClaim, primary: Option<&str>) -> Element {
    let handle = claim.handle.clone().unwrap_or_else(|| "-".to_string());
    let sigil = claim
        .handle
        .as_deref()
        .map(display_sigil)
        .unwrap_or_default();
    let issuer = claim.issuer.clone().unwrap_or_else(|| "-".to_string());
    let expires = claim.expires_at.clone().unwrap_or_else(|| "-".to_string());
    let created = claim.issued_at.clone().unwrap_or_else(|| "-".to_string());
    let (binding_label, binding_variant) = binding_badge(claim.binding_state);
    let is_primary = matches!(
        (claim.handle.as_deref(), primary),
        (Some(h), Some(p)) if h == p
    );
    // "Why am I seeing this?" provenance: issuer DID + binding_state +
    // created_at. Spec §17 derived-projection disclosure rationale.
    let why = format!(
        "{}: {} \u{00b7} {}: {} \u{00b7} {}: {}",
        t("handles_by_subject.why_issuer"),
        issuer,
        t("handles_by_subject.why_binding"),
        binding_label,
        t("handles_by_subject.why_created"),
        created,
    );

    rsx! {
        TableRow {
            TableCell { class: "font-mono text-xs".to_string(),
                div { class: "flex flex-col",
                    div { class: "flex items-center gap-1",
                        span { "{handle}" }
                        if is_primary {
                            Badge { variant: BadgeVariant::Success, class: "text-[10px]".to_string(),
                                {t("handles_by_subject.primary_badge")}
                            }
                        }
                    }
                    if !sigil.is_empty() {
                        span { class: "text-muted-foreground/80 text-[10px]", "{sigil}" }
                    }
                }
            }
            TableCell { class: "font-mono text-xs break-all".to_string(), "{issuer}" }
            TableCell {
                Badge { variant: binding_variant, "{binding_label}" }
            }
            TableCell { class: "font-mono text-xs".to_string(), "{expires}" }
            TableCell {
                span {
                    class: "inline-flex cursor-help items-center text-muted-foreground underline decoration-dotted",
                    title: "{why}",
                    {t("handles_by_subject.why")}
                }
            }
        }
    }
}

fn binding_badge(state: Option<HandleBindingState>) -> (String, BadgeVariant) {
    match state {
        Some(HandleBindingState::Verified) => (
            t("handles_by_subject.binding_verified"),
            BadgeVariant::Success,
        ),
        Some(HandleBindingState::Pending) => (
            t("handles_by_subject.binding_pending"),
            BadgeVariant::Outline,
        ),
        Some(HandleBindingState::Revoked) => (
            t("handles_by_subject.binding_revoked"),
            BadgeVariant::Destructive,
        ),
        Some(HandleBindingState::Expired) => (
            t("handles_by_subject.binding_expired"),
            BadgeVariant::Secondary,
        ),
        None => ("-".to_string(), BadgeVariant::Outline),
    }
}
