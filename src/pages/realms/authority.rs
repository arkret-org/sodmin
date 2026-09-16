//! Read-only view of one Realm's governing authority and commit stream head.
//!
//! Shared Realm state is final only through an authority-signed `RealmCommit`
//! produced by the Realm's single current governing Station. This page shows the
//! three facts an operator needs about that authority: who governs the Realm
//! now, where the Realm commit stream head sits, and the continuous
//! double-signed handoff chain that connects genesis to the current generation.
//!
//! Station replacement is only ever a planned old/new double-signed handoff, so
//! a gap or an unsigned step in this chain is the operator-visible symptom of a
//! Realm whose authority cannot be verified.

use dioxus::prelude::*;

use crate::api::authority;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::card::*;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::utils::i18n::t;

#[component]
pub fn RealmAuthorityPage(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let realm_id_for_fetch = realm_id.clone();
    let mut data = use_resource(move || {
        let id = realm_id_for_fetch.clone();
        async move { authority::get_realm_authority_bundle(&id).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("realm_authority.title").replace("{realm_id}", &realm_id),
                description: t("realm_authority.description"),
            }

            match &*data.read() {
                Some(Ok(bundle)) => {
                    let current_service = bundle.current_service_id.to_string();
                    let current_generation = bundle.current_generation.to_string();
                    let head_position = bundle.realm_stream_head.stream_position.to_string();
                    let head_commit = bundle.realm_stream_head.commit_id.to_string();
                    let genesis_event = bundle.genesis_event.event_id.to_string();
                    let genesis_commit = bundle.genesis_commit.commit_id.to_string();
                    let assertion_nonce = bundle.current_assertion.nonce.as_str().to_owned();
                    let assertion_expires = bundle.current_assertion.expires_at.to_rfc3339();
                    let last_handoff = bundle
                        .current_assertion
                        .last_handoff_ref
                        .as_ref()
                        .map_or_else(|| "-".to_owned(), ToString::to_string);
                    let handoffs = bundle
                        .authority_transitions
                        .iter()
                        .map(HandoffRow::from_transition)
                        .collect::<Vec<_>>();
                    rsx! {
                        Card {
                            CardHeader { CardTitle { {t("realm_authority.current_station_title")} } }
                            CardContent {
                                div { class: "space-y-2 text-sm",
                                    div { span { class: "text-muted-foreground mr-2", {t("realm_authority.station_label")} } span { class: "font-mono text-xs break-all", "{current_service}" } }
                                    div { span { class: "text-muted-foreground mr-2", {t("realm_authority.generation_label")} } span { "{current_generation}" } }
                                    div { span { class: "text-muted-foreground mr-2", {t("realm_authority.last_handoff_label")} } span { class: "font-mono text-xs break-all", "{last_handoff}" } }
                                    div { span { class: "text-muted-foreground mr-2", {t("realm_authority.assertion_nonce_label")} } span { class: "font-mono text-xs break-all", "{assertion_nonce}" } }
                                    div { span { class: "text-muted-foreground mr-2", {t("realm_authority.assertion_expires_label")} } span { "{assertion_expires}" } }
                                }
                            }
                        }

                        Card {
                            CardHeader { CardTitle { {t("realm_authority.stream_head_title")} } }
                            CardContent {
                                div { class: "space-y-2 text-sm",
                                    p { class: "text-muted-foreground", {t("realm_authority.stream_head_note")} }
                                    div { span { class: "text-muted-foreground mr-2", {t("realm_authority.stream_position_label")} } span { "{head_position}" } }
                                    div { span { class: "text-muted-foreground mr-2", {t("realm_authority.head_commit_label")} } span { class: "font-mono text-xs break-all", "{head_commit}" } }
                                    div { span { class: "text-muted-foreground mr-2", {t("realm_authority.genesis_event_label")} } span { class: "font-mono text-xs break-all", "{genesis_event}" } }
                                    div { span { class: "text-muted-foreground mr-2", {t("realm_authority.genesis_commit_label")} } span { class: "font-mono text-xs break-all", "{genesis_commit}" } }
                                }
                            }
                        }

                        Card {
                            CardHeader { CardTitle { {t("realm_authority.handoff_chain_title")} } }
                            CardContent {
                                if handoffs.is_empty() {
                                    EmptyState {
                                        icon_name: "shield".to_string(),
                                        title: t("realm_authority.no_handoff_title"),
                                        description: t("realm_authority.no_handoff_description"),
                                    }
                                } else {
                                    div { class: "rounded-md border overflow-x-auto",
                                        Table {
                                            TableHeader {
                                                TableRow {
                                                    TableHead { {t("realm_authority.col_generations")} }
                                                    TableHead { {t("realm_authority.col_from_station")} }
                                                    TableHead { {t("realm_authority.col_to_station")} }
                                                    TableHead { {t("realm_authority.col_change_commit")} }
                                                    TableHead { {t("realm_authority.col_snapshot")} }
                                                    TableHead { {t("realm_authority.col_signatures")} }
                                                }
                                            }
                                            TableBody {
                                                for row in &handoffs {
                                                    TableRow {
                                                        TableCell { "{row.generations}" }
                                                        TableCell { class: "font-mono text-xs break-all", "{row.from_station}" }
                                                        TableCell { class: "font-mono text-xs break-all", "{row.to_station}" }
                                                        TableCell { class: "font-mono text-xs break-all", "{row.change_commit}" }
                                                        TableCell { class: "font-mono text-xs break-all", "{row.snapshot}" }
                                                        TableCell {
                                                            div { class: "font-mono text-xs break-all", "{row.old_signature}" }
                                                            div { class: "font-mono text-xs break-all", "{row.acceptance_signature}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(error)) => rsx! {
                    ErrorBanner {
                        message: error.message.clone(),
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}

/// One rendered row of the double-signed authority handoff chain.
struct HandoffRow {
    generations: String,
    from_station: String,
    to_station: String,
    change_commit: String,
    snapshot: String,
    old_signature: String,
    acceptance_signature: String,
}

impl HandoffRow {
    fn from_transition(transition: &arkret_wire::RealmAuthorityTransition) -> Self {
        let handoff = &transition.handoff;
        Self {
            generations: format!("{} → {}", handoff.from_generation, handoff.to_generation),
            from_station: handoff.from_service_id.to_string(),
            to_station: handoff.to_service_id.to_string(),
            change_commit: handoff.change_commit_id.to_string(),
            snapshot: handoff.snapshot_ref.to_string(),
            old_signature: handoff
                .old_authority_signature
                .verification_method
                .to_string(),
            acceptance_signature: handoff
                .new_authority_acceptance_signature
                .verification_method
                .to_string(),
        }
    }
}
