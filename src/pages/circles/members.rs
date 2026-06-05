//! `/circles/:circle_id/members` — Circle membership editor.
//!
//! The Circle object comes from `cokret_core::model::Circle` and does not
//! inline a member list. This page therefore exposes explicit add/remove
//! operations keyed by actor DID instead of depending on a local members
//! projection.

use cokret_core::Did;
use dioxus::prelude::*;

use crate::api::circles;
use crate::components::ui::button::Button;
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::types::circles::{CircleMemberRequest, circle_is_active, parse_member_state};
use crate::utils::i18n::t;

#[component]
pub fn CircleMembers(circle_id: String) -> Element {
    let circle_id_for_resource = circle_id.clone();
    let mut data = use_resource(move || {
        let id = circle_id_for_resource.clone();
        async move { circles::get_circle(&id).await }
    });

    let mut new_actor = use_signal(String::new);
    let mut new_state = use_signal(|| "active".to_string());
    let mut remove_actor = use_signal(String::new);
    let mut adding = use_signal(|| false);
    let mut removing = use_signal(|| false);

    let cid_for_add = circle_id.clone();
    let cid_for_remove = circle_id.clone();

    rsx! {
        div { class: "space-y-6",
            match &*data.read() {
                Some(Ok(circle)) => {
                    let title = circle.title.clone();
                    let cid = circle.id.to_string();
                    let realm_id = circle.realm_id.to_string();
                    let is_active = circle_is_active(circle);

                    let breadcrumbs = vec![
                        BreadcrumbItem { label: t("circle.list_title"), route: Some(Route::CircleList {}) },
                        BreadcrumbItem { label: title.clone(), route: Some(Route::CircleShow { circle_id: cid.clone() }) },
                        BreadcrumbItem { label: t("circle.manage_members"), route: None },
                    ];

                    let cid_add = cid_for_add.clone();
                    let cid_remove = cid_for_remove.clone();

                    rsx! {
                        Breadcrumbs { items: breadcrumbs }
                        PageHeader {
                            title: t("circle.manage_members"),
                            description: format!("{}: {}", t("circle.realm_id"), realm_id),
                        }

                        Card {
                            CardHeader { CardTitle { {t("circle.add_member")} } }
                            CardContent {
                                form {
                                    class: "space-y-3",
                                    onsubmit: move |evt: Event<FormData>| {
                                        evt.prevent_default();
                                        let actor = match Did::new(new_actor.read().trim().to_string()) {
                                            Ok(value) => value,
                                            Err(_) => {
                                                show_toast(&t("circle.error_actor_required"), ToastVariant::Error);
                                                return;
                                            }
                                        };
                                        let state = match parse_member_state(new_state.read().trim()) {
                                            Some(value) => value,
                                            None => {
                                                show_toast(&t("circle.error_required"), ToastVariant::Error);
                                                return;
                                            }
                                        };
                                        let req = CircleMemberRequest {
                                            actor_id: actor,
                                            state: Some(state),
                                        };
                                        let cid = cid_add.clone();
                                        adding.set(true);
                                        spawn(async move {
                                            match circles::add_circle_member(&cid, &req).await {
                                                Ok(_) => {
                                                    show_toast(
                                                        &t("circle.member_added_toast"),
                                                        ToastVariant::Success,
                                                    );
                                                    new_actor.set(String::new());
                                                    data.restart();
                                                }
                                                Err(e) => {
                                                    show_toast(&e.message, ToastVariant::Error);
                                                }
                                            }
                                            adding.set(false);
                                        });
                                    },
                                    div { class: "grid gap-3 md:grid-cols-[1fr_180px_auto]",
                                        div { class: "space-y-1",
                                            Label { r#for: "circle-new-member".to_string(), {t("circle.actor_id")} }
                                            Input {
                                                id: "circle-new-member".to_string(),
                                                value: new_actor.read().clone(),
                                                placeholder: "did:web:alice.example".to_string(),
                                                disabled: !is_active,
                                                oninput: move |evt: FormEvent| new_actor.set(evt.value()),
                                            }
                                        }
                                        div { class: "space-y-1",
                                            Label { r#for: "circle-new-state".to_string(), {t("circle.member_state")} }
                                            select {
                                                id: "circle-new-state",
                                                name: "state",
                                                class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                                                disabled: !is_active,
                                                onchange: move |e| new_state.set(e.value()),
                                                value: new_state(),
                                                option { value: "active", "active" }
                                                option { value: "invited", "invited" }
                                            }
                                        }
                                        div { class: "flex items-end",
                                            Button {
                                                r#type: "submit".to_string(),
                                                disabled: adding() || !is_active,
                                                {t("circle.add_member")}
                                            }
                                        }
                                    }
                                    p { class: "text-xs text-muted-foreground",
                                        {t("circle.subset_hint")}
                                    }
                                }
                            }
                        }

                        Card {
                            CardHeader { CardTitle { {t("circle.remove_member")} } }
                            CardContent {
                                form {
                                    class: "space-y-3",
                                    onsubmit: move |evt: Event<FormData>| {
                                        evt.prevent_default();
                                        let actor = match Did::new(remove_actor.read().trim().to_string()) {
                                            Ok(value) => value,
                                            Err(_) => {
                                                show_toast(&t("circle.error_actor_required"), ToastVariant::Error);
                                                return;
                                            }
                                        };
                                        let cid = cid_remove.clone();
                                        removing.set(true);
                                        spawn(async move {
                                            match circles::remove_circle_member(&cid, actor.as_str()).await {
                                                Ok(_) => {
                                                    show_toast(
                                                        &t("circle.member_removed_toast"),
                                                        ToastVariant::Success,
                                                    );
                                                    remove_actor.set(String::new());
                                                    data.restart();
                                                }
                                                Err(e) => show_toast(&e.message, ToastVariant::Error),
                                            }
                                            removing.set(false);
                                        });
                                    },
                                    div { class: "grid gap-3 md:grid-cols-[1fr_auto]",
                                        div { class: "space-y-1",
                                            Label { r#for: "circle-remove-member".to_string(), {t("circle.actor_id")} }
                                            Input {
                                                id: "circle-remove-member".to_string(),
                                                value: remove_actor.read().clone(),
                                                placeholder: "did:web:alice.example".to_string(),
                                                disabled: !is_active,
                                                oninput: move |evt: FormEvent| remove_actor.set(evt.value()),
                                            }
                                        }
                                        div { class: "flex items-end",
                                            Button {
                                                r#type: "submit".to_string(),
                                                disabled: removing() || !is_active,
                                                {t("circle.remove_member")}
                                            }
                                        }
                                    }
                                }
                            }
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
