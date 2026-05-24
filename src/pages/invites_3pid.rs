//! Round 4 — 3PID invite state-machine admin view.
//!
//! Renders coauth's third-party-invite (`cx.schema.invite.v1`
//! third_party_invite) admin listing as a state-machine table where
//! every row shows its current terminal state. The five round-4
//! terminal states are:
//!
//! - `claimed`
//! - `send_failed`
//! - `revoked_by_capability_loss`
//! - `revoked_by_inviter_left`
//! - `invalidated_by_rate_limit`
//!
//! Wire-correctness invariants enforced by this page:
//!
//! 1. **No plaintext 3PID.** The wire (and therefore the row) never
//!    carries the email / phone number. We only display opaque evidence
//!    (token_commitment digest / lookup_table_ref / pepper_id).
//! 2. **`send_failed` displayed truthfully.** This is a *terminal*
//!    failure (the auth server never delivered the OOB code). The UI
//!    must surface it as a hard failure and must NOT collapse it into
//!    a generic "pending" or "completed" bucket — papering over
//!    `send_failed` would be lying to the operator about a permanent
//!    delivery failure.
//!
use dioxus::prelude::*;

use crate::api::invites_3pid;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::types::{ThirdPartyInviteRow, ThirdPartyInviteTerminalState};
use crate::utils::i18n::t;

#[component]
pub fn ThirdPartyInvitesPage() -> Element {
    let mut rows_data = use_resource(|| async { invites_3pid::list_third_party_invites().await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("invites_3pid.title"),
                description: t("invites_3pid.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| rows_data.restart(),
                    {t("common.refresh")}
                }
            }

            // Round 4 — the `send_failed` terminal state is permanent
            // and must NOT be collapsed into a generic success.
            // Surface a banner explaining the state-machine semantics
            // so operators reading the table understand it.
            div {
                class: "rounded-md border border-amber-600/40 bg-amber-600/10 p-3 text-xs",
                role: "note",
                p { class: "font-semibold text-amber-700 dark:text-amber-200",
                    {t("invites_3pid.send_failed_warning")}
                }
            }

            Card {
                CardContent {
                    match &*rows_data.read() {
                        Some(Ok(resp)) => rsx! {
                            if resp.data.is_empty() {
                                p { class: "py-8 text-center text-sm text-muted-foreground",
                                    {t("invites_3pid.empty")}
                                }
                            } else {
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("invites_3pid.invite_id")} }
                                            TableHead { {t("invites_3pid.oob_mode")} }
                                            TableHead { {t("invites_3pid.verifier")} }
                                            TableHead { {t("invites_3pid.terminal_state")} }
                                            TableHead { {t("invites_3pid.evidence")} }
                                        }
                                    }
                                    TableBody {
                                        for row in resp.data.iter() {
                                            {render_invite_row(row)}
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(err)) => rsx! {
                            div { class: "rounded-md border border-destructive/40 bg-destructive/10 p-3 text-sm",
                                p { class: "font-semibold", "Failed to load third-party invites" }
                                p { class: "font-mono text-xs", "{err}" }
                            }
                        },
                        None => rsx! {
                            p { class: "py-8 text-center text-sm text-muted-foreground", "Loading..." }
                        },
                    }
                }
            }
        }
    }
}

fn render_invite_row(row: &ThirdPartyInviteRow) -> Element {
    let invite_id = row.invite_id.clone();
    let oob_mode = row
        .oob_code_kind
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let verifier = row
        .verification_service_did
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let evidence = row.evidence.clone().unwrap_or_else(|| "-".to_string());

    let (state_label, state_variant, row_class) = match row.classified_state() {
        Some(ThirdPartyInviteTerminalState::Claimed) => (
            t("invites_3pid.state_claimed"),
            BadgeVariant::Success,
            "",
        ),
        Some(ThirdPartyInviteTerminalState::SendFailed) => (
            t("invites_3pid.state_send_failed"),
            BadgeVariant::Destructive,
            "bg-red-600/5",
        ),
        Some(ThirdPartyInviteTerminalState::RevokedByCapabilityLoss) => (
            t("invites_3pid.state_revoked_by_capability_loss"),
            BadgeVariant::Destructive,
            "",
        ),
        Some(ThirdPartyInviteTerminalState::RevokedByInviterLeft) => (
            t("invites_3pid.state_revoked_by_inviter_left"),
            BadgeVariant::Destructive,
            "",
        ),
        Some(ThirdPartyInviteTerminalState::InvalidatedByRateLimit) => (
            t("invites_3pid.state_invalidated_by_rate_limit"),
            BadgeVariant::Destructive,
            "",
        ),
        None => (
            row.terminal_state.clone().unwrap_or_else(|| "-".to_string()),
            BadgeVariant::Secondary,
            "",
        ),
    };

    rsx! {
        TableRow { class: row_class.to_string(),
            TableCell { class: "font-mono text-xs".to_string(), "{invite_id}" }
            TableCell { class: "font-mono text-xs".to_string(), "{oob_mode}" }
            TableCell { class: "font-mono text-xs break-all".to_string(), "{verifier}" }
            TableCell {
                Badge { variant: state_variant, class: "font-mono text-xs".to_string(), "{state_label}" }
            }
            TableCell { class: "font-mono text-xs break-all".to_string(), "{evidence}" }
        }
    }
}

/// Fixture covering every round-4 terminal state. Real data comes from
/// the coauth admin listing once it ships.
fn placeholder_third_party_invites() -> Vec<ThirdPartyInviteRow> {
    ThirdPartyInviteTerminalState::all()
        .into_iter()
        .enumerate()
        .map(|(i, state)| ThirdPartyInviteRow {
            invite_id: format!("inv-{:02}", i + 1),
            oob_code_kind: Some(if i % 2 == 0 {
                "offline_token".into()
            } else {
                "lookup".into()
            }),
            verification_service_did: Some(format!("did:web:auth.example/{}", i)),
            terminal_state: Some(state.slug().into()),
            evidence: Some(match state {
                ThirdPartyInviteTerminalState::Claimed
                | ThirdPartyInviteTerminalState::SendFailed => {
                    "token_commitment=sha256:0123..deadbeef".into()
                }
                ThirdPartyInviteTerminalState::RevokedByCapabilityLoss
                | ThirdPartyInviteTerminalState::RevokedByInviterLeft => {
                    "lookup_table_ref=lt:beta pepper_id=p-42".into()
                }
                ThirdPartyInviteTerminalState::InvalidatedByRateLimit => {
                    "lookup_table_ref=lt:beta pepper_id=p-43 (3 errors)".into()
                }
            }),
            observed_at: Some("2026-05-20T10:00:00Z".into()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_covers_all_five_terminal_states() {
        let rows = placeholder_third_party_invites();
        let states: Vec<_> = rows.iter().filter_map(|r| r.classified_state()).collect();
        assert!(states.contains(&ThirdPartyInviteTerminalState::Claimed));
        assert!(states.contains(&ThirdPartyInviteTerminalState::SendFailed));
        assert!(states.contains(&ThirdPartyInviteTerminalState::RevokedByCapabilityLoss));
        assert!(states.contains(&ThirdPartyInviteTerminalState::RevokedByInviterLeft));
        assert!(states.contains(&ThirdPartyInviteTerminalState::InvalidatedByRateLimit));
    }

    #[test]
    fn placeholder_rows_never_carry_plaintext_3pid() {
        // The plaintext email / SMS MUST NEVER appear in the admin
        // view — the wire doesn't carry it, and reconstruction is
        // explicitly out of scope.
        let rows = placeholder_third_party_invites();
        for row in rows {
            let evidence = row.evidence.unwrap_or_default();
            assert!(!evidence.contains('@'), "evidence leaks an email: {evidence}");
            // Sanity: a 10+ digit phone number would be 10+ contiguous
            // digits; the placeholder fixture must not include any.
            let digits: String = evidence.chars().filter(|c| c.is_ascii_digit()).collect();
            assert!(digits.len() < 10, "evidence has many digits: {digits}");
        }
    }
}
