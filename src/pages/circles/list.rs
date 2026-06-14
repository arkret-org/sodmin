//! `/circles` — list Circles inside a single Realm.
//!
//! Circles are scoped to a Realm, so the list view needs a Realm
//! selector (a free-text `realm_id` field for now — once the Realm
//! directory ships a per-Realm picker component, swap that in).
//! Empty Realms render the shared `EmptyState` with a deep-link to the
//! create form pre-filled with the realm_id.

use dioxus::prelude::*;

use crate::api::circles;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::icons::Icon;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::router::Route;
use crate::types::circles::{Circle, circle_state_label_key};
use crate::utils::i18n::t;

#[component]
pub fn CircleList() -> Element {
    let mut realm_input = use_signal(String::new);
    let mut applied_realm = use_signal(String::new);

    let realm_snapshot = applied_realm.read().clone();
    let mut data = use_resource(move || {
        let realm = realm_snapshot.clone();
        async move {
            if realm.trim().is_empty() {
                None
            } else {
                Some(circles::list_circles(&realm).await)
            }
        }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "My Circles".to_string(),
                description: "Self-scoped Circle operations for the authenticated principal.".to_string(),
                Link {
                    to: Route::CircleCreate {},
                    class: "inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90",
                    Icon { name: "plus".to_string(), class: "h-4 w-4".to_string() }
                    {t("circle.create")}
                }
            }

            div { class: "rounded-lg border glass-panel p-4 space-y-3",
                div { class: "space-y-1",
                    Label { r#for: "circle-realm-filter".to_string(), {t("circle.filter_realm")} }
                    div { class: "flex gap-2",
                        Input {
                            id: "circle-realm-filter".to_string(),
                            value: realm_input.read().clone(),
                            placeholder: "ck:realm:01H...".to_string(),
                            oninput: move |evt: FormEvent| realm_input.set(evt.value()),
                        }
                        Button {
                            variant: ButtonVariant::Default,
                            onclick: move |_| {
                                applied_realm.set(realm_input.read().clone());
                            },
                            {t("circle.filter_apply")}
                        }
                    }
                    p { class: "text-xs text-muted-foreground",
                        {t("circle.filter_realm_hint")}
                    }
                }
            }

            if applied_realm.read().trim().is_empty() {
                EmptyState {
                    icon_name: "users".to_string(),
                    title: t("circle.empty_no_realm_title"),
                    description: t("circle.empty_no_realm_description"),
                }
            } else {
                match &*data.read() {
                    Some(Some(Ok(resp))) => rsx! {
                        if resp.circles.is_empty() {
                            EmptyState {
                                icon_name: "users".to_string(),
                                title: t("circle.empty_title"),
                                description: t("circle.empty_description"),
                            }
                        } else {
                            div { class: "rounded-lg border glass-panel overflow-hidden",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("circle.id")} }
                                            TableHead { {t("circle.title")} }
                                            TableHead { {t("circle.realm_id")} }
                                            TableHead { {t("circle.state")} }
                                            TableHead { {t("circle.created_at")} }
                                        }
                                    }
                                    TableBody {
                                        for c in resp.circles.iter() {
                                            {circle_row(c)}
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(Some(Err(e))) => rsx! {
                        ErrorBanner {
                            message: e.message.clone(),
                            on_retry: move |_| data.restart(),
                        }
                    },
                    Some(None) | None => rsx! { PageSkeleton {} },
                }
            }
        }
    }
}

fn circle_row(c: &Circle) -> Element {
    let cid = c.circle_id.to_string();
    let rid = c.realm_id.to_string();
    let (variant, label_key) = state_badge(&c.state);
    let created_at = c.created_at.to_rfc3339();
    rsx! {
        TableRow {
            TableCell { class: "font-mono text-xs".to_string(),
                Link {
                    to: Route::CircleShow { circle_id: cid.clone() },
                    class: "hover:underline",
                    "{cid}"
                }
            }
            TableCell { "{c.title}" }
            TableCell { class: "font-mono text-xs".to_string(), "{rid}" }
            TableCell {
                Badge { variant, {t(label_key)} }
            }
            TableCell { class: "text-muted-foreground".to_string(), "{created_at}" }
        }
    }
}

/// Map a `ck.circle.state` value to a badge variant + i18n key.
pub(crate) fn state_badge(state: &cokret_core::model::CircleState) -> (BadgeVariant, &'static str) {
    match state {
        cokret_core::model::CircleState::Active => {
            (BadgeVariant::Success, circle_state_label_key(state))
        }
        cokret_core::model::CircleState::Archived => {
            (BadgeVariant::Secondary, circle_state_label_key(state))
        }
        cokret_core::model::CircleState::Tombstoned => {
            (BadgeVariant::Destructive, circle_state_label_key(state))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_badge_picks_correct_variant() {
        use cokret_core::model::CircleState;

        assert!(matches!(
            state_badge(&CircleState::Active).0,
            BadgeVariant::Success
        ));
        assert!(matches!(
            state_badge(&CircleState::Archived).0,
            BadgeVariant::Secondary
        ));
        assert!(matches!(
            state_badge(&CircleState::Tombstoned).0,
            BadgeVariant::Destructive
        ));
    }
}
