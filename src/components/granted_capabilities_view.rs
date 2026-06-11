//! P5 — granted-capabilities preview surface.
//!
//! Renders the current user's coauth-granted capability list before the
//! operator initiates a destructive action. Purpose: avoid the "click ->
//! 403 surprise" loop where an operator drives a destructive form and
//! only discovers their role lacks the scope after submission.
//!
//! The view filters the list by a `required_capability` prop so the
//! parent can pass in e.g. `ck.self.agent.deactivate` and the view will
//! highlight whether that scope is currently granted to the operator.
//!
//! Backend RBAC is canonical — the UI "hiding" of a button is cosmetic
//! only and the server MUST reject any disallowed action by `errcode =
//! capability_denied`. This view is a UX-affordance, not a
//! permission gate.
//!
//! TODO(P5-impl): once coauth ships a typed `/_soland/admin/me/grants`
//! endpoint, swap the current `list_capabilities` heuristic (which lists
//! every grant on the server) for the scoped self-grant call. Until
//! then we filter the list client-side by viewer subject.

use dioxus::prelude::*;

use crate::api::{capabilities, coauth};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::loading::PageSkeleton;
use crate::types::{CapabilityGrant, CapabilityGrantExt};

const VIEW_PAGE_SIZE: u64 = 100;

#[derive(Props, Clone, PartialEq)]
pub struct GrantedCapabilitiesViewProps {
    /// The capability scope the parent is about to exercise — e.g.
    /// `ck.self.agent.deactivate`. The view will surface a prominent banner
    /// when this scope is NOT found in the operator's grant list so
    /// the destructive action button can be visually demoted.
    #[props(default)]
    pub required_capability: Option<String>,
    /// Optional title override. Defaults to "Your granted capabilities".
    #[props(default)]
    pub title: Option<String>,
}

#[component]
pub fn GrantedCapabilitiesView(props: GrantedCapabilitiesViewProps) -> Element {
    let required = props.required_capability.clone();
    let title = props
        .title
        .clone()
        .unwrap_or_else(|| "Your granted capabilities".to_string());

    let viewer_data = use_resource(|| async move { coauth::get_viewer().await });
    let grants_data =
        use_resource(|| async move { capabilities::list_capabilities(1, VIEW_PAGE_SIZE).await });

    let viewer_sub: Option<String> = match &*viewer_data.read() {
        Some(Ok(v)) => Some(v.sub.clone()),
        _ => None,
    };

    rsx! {
        div { class: "rounded-md border p-4 space-y-3 bg-muted/30",
            h3 { class: "text-sm font-semibold", "{title}" }

            match &*grants_data.read() {
                Some(Ok(resp)) => {
                    let viewer = viewer_sub.clone();
                    let self_grants: Vec<&CapabilityGrant> = resp
                        .data
                        .iter()
                        .filter(|g| {
                            !g.is_revoked()
                                && viewer
                                    .as_ref()
                                    .map(|sub| g.subject_display() == *sub)
                                    .unwrap_or(true)
                        })
                        .collect();

                    let scope_match = required
                        .as_ref()
                        .map(|cap| self_grants.iter().any(|g| g.actions.iter().any(|action| action == cap)))
                        .unwrap_or(true);

                    rsx! {
                        if let Some(cap) = required.clone() {
                            if scope_match {
                                p { class: "text-xs text-foreground",
                                    "Required scope "
                                    code { class: "font-mono", "{cap}" }
                                    " is granted to you. The backend RBAC layer will still re-check."
                                }
                            } else {
                                p { class: "text-xs text-destructive font-medium",
                                    "Required scope "
                                    code { class: "font-mono", "{cap}" }
                                    " is NOT in your current grant list. Backend RBAC will reject this action with capability_denied."
                                }
                            }
                        }

                        if self_grants.is_empty() {
                            p { class: "text-xs text-muted-foreground",
                                "No active capabilities found for your account. UI \"hiding\" of buttons is cosmetic only — server-side RBAC is canonical."
                            }
                        } else {
                            ul { class: "flex flex-wrap gap-1.5",
                                for grant in self_grants.iter() {
                                    for action in grant.actions.iter() {
                                        li {
                                            Badge {
                                                variant: if required.as_ref().map(|r| r == action).unwrap_or(false) {
                                                    BadgeVariant::Default
                                                } else {
                                                    BadgeVariant::Secondary
                                                },
                                                "{action}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! {
                    p { class: "text-xs text-muted-foreground",
                        "Could not load grant list: "
                        span { class: "font-mono", "{e.message}" }
                        ". Backend RBAC will still enforce on submit."
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
