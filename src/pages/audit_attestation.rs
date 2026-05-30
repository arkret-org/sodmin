//! Audit attestation_evidence admin page.
//!
//! Operators of an `attested_audit.e2ee.v1` deployment join Audit Agents
//! into the Realm by anchoring `cx.schema.attestation_evidence.v1` rows
//! that bind the agent's MLS leaf key to a remote-attestation chain
//! (SGX / TDX / SEV-SNP / TPM2). This admin page lets the operator:
//!
//! - submit a new evidence document (JSON pasted into the form, or
//!   uploaded as a file)
//! - browse existing rows
//! - see the validity window remaining in days
//! - see the chain-verification + revocation status badges
//!
//! Full chain verification visualization (per-cert, per-revocation
//! endpoint) is not wired yet. The verification chips render the
//! principal server's coarse `{chain_verified, revocation_checked}`
//! flags.
//!
//! TODO(circle-rollout-P3A.5): once attestation evidence rows surface
//! their pinning Circle / Realm in the wire shape, add an
//! `effective_scope` column here too. Today the rows are scoped to the
//! deployment, not to a specific Circle, so the column would be empty.

use dioxus::prelude::*;

use crate::api::server;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::ListResponse;

/// Coarse status returned by the principal server for a stored
/// attestation evidence row. Real `cx.schema.attestation_evidence.v1`
/// has a much richer chain shape — this is the admin projection.
#[derive(Debug, Clone, serde::Deserialize, Default)]
struct AttestationRow {
    #[serde(default)]
    evidence_id: String,
    #[serde(default)]
    audit_agent_did: String,
    #[serde(default)]
    platform_family: String,
    /// Days remaining until `validity.not_after` lapses. Negative when
    /// expired.
    #[serde(default)]
    validity_remaining_days: i64,
    #[serde(default)]
    chain_verified: bool,
    #[serde(default)]
    revocation_checked: bool,
}

async fn list_attestation_rows()
-> Result<ListResponse<AttestationRow>, crate::utils::error::HttpError> {
    crate::api::client::api_client("/api/admin/v1/audit/attestation-evidence", "GET", None).await
}

#[component]
pub fn AuditAttestationPage() -> Element {
    let mut draft = use_signal(String::new);
    let mut parse_error = use_signal::<Option<String>>(|| None);
    let mut rows_data = use_resource(|| async move { list_attestation_rows().await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Audit attestation evidence".to_string(),
                description: "Submit and review `cx.schema.attestation_evidence.v1` documents binding Audit Agents to a remote-attestation chain.".to_string(),
            }

            Card {
                CardHeader {
                    CardTitle { class: "text-lg".to_string(), "Submit new evidence" }
                    CardDescription {
                        "Paste a JSON document matching the `cx.schema.attestation_evidence.v1` schema. The principal server re-validates the chain and the MLS leaf binding before anchoring."
                    }
                }
                CardContent { class: "space-y-3".to_string(),
                    textarea {
                        class: "w-full min-h-[180px] rounded-md border border-input bg-background p-2 font-mono text-xs",
                        placeholder: "{{\n  \"evidence_id\": \"att:...\",\n  \"audit_agent_did\": \"did:web:...\",\n  \"platform\": {{\"family\": \"tee_sgx\", ...}},\n  ...\n}}",
                        value: draft.read().clone(),
                        oninput: move |evt| {
                            parse_error.set(None);
                            draft.set(evt.value());
                        },
                    }
                    if let Some(err) = parse_error.read().as_ref() {
                        p { class: "text-xs text-destructive font-mono", "{err}" }
                    }
                    div { class: "flex justify-end gap-2",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                draft.set(String::new());
                                parse_error.set(None);
                            },
                            "Clear"
                        }
                        Button {
                            variant: ButtonVariant::Default,
                            onclick: move |_| {
                                let body = draft.read().clone();
                                if body.trim().is_empty() {
                                    parse_error.set(Some("body is empty".to_string()));
                                    return;
                                }
                                // Pre-flight: ensure the body is valid
                                // JSON. The principal server does the
                                // real schema validation; surfacing the
                                // structural error here is just a UX
                                // nicety so admins don't ship
                                // unparseable strings.
                                match serde_json::from_str::<serde_json::Value>(&body) {
                                    Ok(value) => {
                                        spawn(async move {
                                            match server::submit_attestation_evidence(&value).await {
                                                Ok(_) => show_toast(
                                                    "Attestation evidence submitted.",
                                                    ToastVariant::Success,
                                                ),
                                                Err(err) => show_toast(
                                                    &format!("Attestation evidence submit failed: {err}"),
                                                    ToastVariant::Error,
                                                ),
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        parse_error.set(Some(format!("JSON parse error: {e}")));
                                    }
                                }
                            },
                            "Submit"
                        }
                    }
                }
            }

            Card {
                CardHeader {
                    CardTitle { class: "text-lg".to_string(), "Active evidence rows" }
                    CardDescription {
                        "Validity window + chain / revocation status per row. No synthetic rows are rendered."
                    }
                }
                CardContent {
                    match &*rows_data.read() {
                        Some(Ok(resp)) => if resp.data.is_empty() {
                            rsx! {
                                EmptyState {
                                    icon: "shield".to_string(),
                                    title: "No attestation evidence rows".to_string(),
                                    description: "The principal server returned an empty attestation evidence list.".to_string(),
                                }
                            }
                        } else {
                            rsx! {
                                ul { class: "space-y-2",
                                    for row in resp.data.iter() {
                                        {evidence_row_card(row)}
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            ErrorBanner {
                                message: format!("Attestation evidence list unavailable: {}", e.message),
                                on_retry: move |_| rows_data.restart(),
                            }
                        },
                        None => rsx! { PageSkeleton {} },
                    }
                }
            }
        }
    }
}

fn evidence_row_card(row: &AttestationRow) -> Element {
    let validity_variant = validity_chip_variant(row.validity_remaining_days);
    let validity_label = validity_chip_label(row.validity_remaining_days);
    let chain_variant = if row.chain_verified {
        BadgeVariant::Success
    } else {
        BadgeVariant::Destructive
    };
    let chain_label = if row.chain_verified {
        "chain verified"
    } else {
        "chain unverified"
    };
    let revocation_variant = if row.revocation_checked {
        BadgeVariant::Success
    } else {
        BadgeVariant::Secondary
    };
    let revocation_label = if row.revocation_checked {
        "revocation checked"
    } else {
        "revocation pending"
    };
    rsx! {
        li { class: "rounded-md border p-3 space-y-2",
            div { class: "flex flex-wrap items-start justify-between gap-2",
                div { class: "space-y-1",
                    p { class: "font-mono text-xs break-all", "{row.evidence_id}" }
                    p { class: "font-mono text-xs text-muted-foreground", "{row.audit_agent_did}" }
                    p { class: "font-mono text-[10px] uppercase tracking-wider text-muted-foreground", "{row.platform_family}" }
                }
                div { class: "flex flex-wrap gap-1",
                    Badge { variant: validity_variant, "{validity_label}" }
                    Badge { variant: chain_variant, "{chain_label}" }
                    Badge { variant: revocation_variant, "{revocation_label}" }
                }
            }
        }
    }
}

/// Red if expired, amber-like (Destructive) if within 7 days, green
/// otherwise. The principal-server reducer rejects evidence whose
/// `validity.not_after <= now`.
fn validity_chip_variant(days: i64) -> BadgeVariant {
    if days <= 7 {
        BadgeVariant::Destructive
    } else {
        BadgeVariant::Success
    }
}

fn validity_chip_label(days: i64) -> String {
    if days < 0 {
        "expired".to_string()
    } else if days == 0 {
        "expires today".to_string()
    } else if days == 1 {
        "1 day remaining".to_string()
    } else {
        format!("{days} days remaining")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validity_chip_handles_negative_zero_short_long() {
        assert_eq!(validity_chip_label(-5), "expired");
        assert_eq!(validity_chip_label(0), "expires today");
        assert_eq!(validity_chip_label(1), "1 day remaining");
        assert_eq!(validity_chip_label(14), "14 days remaining");

        assert!(matches!(
            validity_chip_variant(-1),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            validity_chip_variant(7),
            BadgeVariant::Destructive
        ));
        assert!(matches!(validity_chip_variant(8), BadgeVariant::Success));
    }
}
