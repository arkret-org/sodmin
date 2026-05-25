//! `/circles/:circle_id/scope` — MLS scope rotation surface.
//!
//! CXP-0007 makes scope rotation a first-class admin operation so the
//! receipt fans out into the audit log even when no membership changes
//! accompany it. The full MLS-key cascade lives in soland's
//! `reducer/mls.rs` and is still
//! `TODO(circle-rollout-P2A.4)`; here we expose the trigger plus the
//! current `mls_group_ref`.

use dioxus::prelude::*;

use crate::api::circles;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::utils::i18n::t;

#[component]
pub fn CircleScope(circle_id: String) -> Element {
    let circle_id_for_resource = circle_id.clone();
    let mut data = use_resource(move || {
        let id = circle_id_for_resource.clone();
        async move { circles::get_circle(&id).await }
    });

    let mut last_note = use_signal(String::new);
    let mut rotating = use_signal(|| false);

    let cid_for_rotate = circle_id.clone();

    rsx! {
        div { class: "space-y-6",
            match &*data.read() {
                Some(Ok(circle)) => {
                    let title = circle.title.clone();
                    let cid = circle.circle_id.clone();
                    let mls_group_ref = circle.mls_group_ref.clone().unwrap_or_else(|| "-".to_string());
                    let encryption_profile = circle.encryption_profile.clone();
                    let is_active = circle.is_active();

                    let breadcrumbs = vec![
                        BreadcrumbItem { label: t("circle.list_title"), route: Some(Route::CircleList {}) },
                        BreadcrumbItem { label: title.clone(), route: Some(Route::CircleShow { circle_id: cid.clone() }) },
                        BreadcrumbItem { label: t("circle.manage_scope"), route: None },
                    ];

                    let cid_rotate = cid_for_rotate.clone();

                    rsx! {
                        Breadcrumbs { items: breadcrumbs }
                        PageHeader {
                            title: t("circle.manage_scope"),
                            description: t("circle.scope_description"),
                        }

                        Card {
                            CardHeader { CardTitle { {t("circle.encryption_profile")} } }
                            CardContent {
                                div { class: "space-y-3",
                                    {field_row(t("circle.encryption_profile"), encryption_profile)}
                                    {field_row(t("circle.mls_group_ref"), mls_group_ref)}
                                    if !last_note.read().is_empty() {
                                        div { class: "rounded-md border border-amber-500/30 bg-amber-500/10 p-3 text-sm",
                                            "{last_note}"
                                        }
                                    }
                                    Button {
                                        variant: ButtonVariant::Default,
                                        disabled: rotating() || !is_active,
                                        onclick: move |_| {
                                            let cid = cid_rotate.clone();
                                            rotating.set(true);
                                            spawn(async move {
                                                match circles::rotate_circle_scope(&cid).await {
                                                    Ok(resp) => {
                                                        show_toast(
                                                            &t("circle.scope_rotated_toast"),
                                                            ToastVariant::Success,
                                                        );
                                                        if let Some(note) = resp.note {
                                                            last_note.set(note);
                                                        }
                                                        data.restart();
                                                    }
                                                    Err(e) => show_toast(&e.message, ToastVariant::Error),
                                                }
                                                rotating.set(false);
                                            });
                                        },
                                        {t("circle.rotate_scope")}
                                    }
                                    p { class: "text-xs text-muted-foreground",
                                        {t("circle.scope_hint")}
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

fn field_row(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex justify-between py-1.5 border-b border-border/50 last:border-0",
            span { class: "text-sm text-muted-foreground", "{label}" }
            span { class: "text-sm font-medium font-mono", "{value}" }
        }
    }
}
