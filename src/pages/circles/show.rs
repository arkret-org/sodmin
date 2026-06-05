//! `/circles/:circle_id` — Circle detail + lifecycle action panel.
//!
//! Surfaces the spec's full Circle projection (title, summary, MLS group
//! ref, encryption profile, member count, lifecycle state) and exposes
//! the archive / tombstone / scope-rotate / member-list actions in a
//! single panel. Archive and tombstone are gated by `ConfirmDialog`
//! because tombstone is terminal and archive cuts off ordinary writes.

use dioxus::prelude::*;

use crate::api::circles;
use crate::components::ui::badge::Badge;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::pages::circles::list::state_badge;
use crate::router::Route;
use crate::types::circles::{
    circle_is_active, circle_is_tombstoned, directory_visibility_wire, encryption_profile_wire,
    history_visibility_wire, join_rule_wire,
};
use crate::utils::i18n::t;

#[component]
pub fn CircleShow(circle_id: String) -> Element {
    let circle_id_for_resource = circle_id.clone();
    let mut data = use_resource(move || {
        let id = circle_id_for_resource.clone();
        async move { circles::get_circle(&id).await }
    });

    let mut confirm_archive = use_signal(|| false);
    let mut confirm_tombstone = use_signal(|| false);

    rsx! {
        div { class: "space-y-6",
            match &*data.read() {
                Some(Ok(circle)) => {
                    let cid = circle.id.to_string();
                    let realm_id = circle.realm_id.to_string();
                    let title = circle.title.clone();
                    let (variant, label_key) = state_badge(&circle.state);
                    let is_active = circle_is_active(circle);
                    let is_terminal = circle_is_tombstoned(circle);
                    let summary = circle.summary.clone();
                    let encryption_profile = encryption_profile_wire(&circle.encryption_profile).to_string();
                    let mls_group_ref = circle.mls_group_ref.clone();
                    let created_by = circle.created_by.to_string();
                    let created_at = circle.created_at.to_rfc3339();
                    let updated_at = circle.updated_at.as_ref().map(|dt| dt.to_rfc3339());
                    let directory_visibility =
                        directory_visibility_wire(&circle.directory_visibility).to_string();
                    let join_rule = join_rule_wire(&circle.join_rule).to_string();
                    let history_visibility =
                        history_visibility_wire(&circle.history_visibility).to_string();

                    let breadcrumbs = vec![
                        BreadcrumbItem { label: t("circle.list_title"), route: Some(Route::CircleList {}) },
                        BreadcrumbItem { label: title.clone(), route: None },
                    ];

                    let cid_archive = cid.clone();
                    let cid_tombstone = cid.clone();
                    let cid_scope = cid.clone();
                    let cid_members = cid.clone();
                    let cid_scope_link = cid.clone();

                    rsx! {
                        Breadcrumbs { items: breadcrumbs }
                        PageHeader {
                            title: title.clone(),
                            description: format!("{}: {}", t("circle.id"), cid),
                            Badge { variant, {t(label_key)} }
                        }

                        div { class: "grid gap-6 md:grid-cols-2",
                            Card {
                                CardHeader { CardTitle { {t("circle.overview")} } }
                                CardContent {
                                    div { class: "space-y-3",
                                        {field_row(t("circle.id"), cid.clone())}
                                        {field_row(t("circle.realm_id"), realm_id.clone())}
                                        {field_row(t("circle.directory_visibility"), directory_visibility)}
                                        {field_row(t("circle.join_rule"), join_rule)}
                                        {field_row(t("circle.history_visibility"), history_visibility)}
                                        {field_row(t("circle.encryption_profile"), encryption_profile)}
                                        {field_row(
                                            t("circle.mls_group_ref"),
                                            mls_group_ref.clone().unwrap_or_else(|| "-".to_string()),
                                        )}
                                        {field_row(t("circle.created_by"), created_by)}
                                        {field_row(t("circle.created_at"), created_at)}
                                        {field_row(
                                            t("circle.updated_at"),
                                            updated_at.unwrap_or_else(|| "-".to_string()),
                                        )}
                                        if let Some(s) = summary {
                                            div { class: "pt-2",
                                                p { class: "text-sm text-muted-foreground mb-1", {t("circle.summary")} }
                                                p { class: "text-sm", "{s}" }
                                            }
                                        }
                                    }
                                }
                            }

                            Card {
                                CardHeader { CardTitle { {t("circle.actions")} } }
                                CardContent {
                                    div { class: "space-y-2",
                                        Link {
                                            to: Route::CircleMembers { circle_id: cid_members.clone() },
                                            class: "inline-flex h-10 w-full items-center justify-center rounded-md border px-3 py-2 text-sm hover:bg-accent",
                                            {t("circle.manage_members")}
                                        }
                                        Link {
                                            to: Route::CircleScope { circle_id: cid_scope_link.clone() },
                                            class: "inline-flex h-10 w-full items-center justify-center rounded-md border px-3 py-2 text-sm hover:bg-accent",
                                            {t("circle.manage_scope")}
                                        }
                                        Button {
                                            class: "w-full".to_string(),
                                            variant: ButtonVariant::Secondary,
                                            disabled: !is_active,
                                            onclick: move |_| {
                                                let cid = cid_scope.clone();
                                                spawn(async move {
                                                    match circles::rotate_circle_scope(&cid).await {
                                                        Ok(_) => show_toast(
                                                            &t("circle.scope_rotated_toast"),
                                                            ToastVariant::Success,
                                                        ),
                                                        Err(e) => show_toast(&e.message, ToastVariant::Error),
                                                    }
                                                });
                                            },
                                            {t("circle.rotate_scope")}
                                        }
                                        Button {
                                            class: "w-full".to_string(),
                                            variant: ButtonVariant::Outline,
                                            disabled: !is_active,
                                            onclick: move |_| confirm_archive.set(true),
                                            {t("circle.archive")}
                                        }
                                        Button {
                                            class: "w-full".to_string(),
                                            variant: ButtonVariant::Destructive,
                                            disabled: is_terminal,
                                            onclick: move |_| confirm_tombstone.set(true),
                                            {t("circle.tombstone")}
                                        }
                                    }
                                }
                            }
                        }
                        ConfirmDialog {
                            open: *confirm_archive.read(),
                            title: t("circle.confirm_archive_title"),
                            description: t("circle.confirm_archive_body"),
                            confirm_text: t("circle.archive"),
                            cancel_text: t("common.cancel"),
                            destructive: false,
                            on_cancel: move |_| confirm_archive.set(false),
                            on_confirm: move |_| {
                                confirm_archive.set(false);
                                let cid = cid_archive.clone();
                                spawn(async move {
                                    match circles::archive_circle(&cid).await {
                                        Ok(_) => {
                                            show_toast(&t("circle.archived_toast"), ToastVariant::Success);
                                            data.restart();
                                        }
                                        Err(e) => show_toast(&e.message, ToastVariant::Error),
                                    }
                                });
                            },
                        }

                        ConfirmDialog {
                            open: *confirm_tombstone.read(),
                            title: t("circle.confirm_tombstone_title"),
                            description: t("circle.confirm_tombstone_body"),
                            confirm_text: t("circle.tombstone"),
                            cancel_text: t("common.cancel"),
                            destructive: true,
                            on_cancel: move |_| confirm_tombstone.set(false),
                            on_confirm: move |_| {
                                confirm_tombstone.set(false);
                                let cid = cid_tombstone.clone();
                                spawn(async move {
                                    match circles::tombstone_circle(&cid).await {
                                        Ok(_) => {
                                            show_toast(&t("circle.tombstoned_toast"), ToastVariant::Success);
                                            data.restart();
                                        }
                                        Err(e) => show_toast(&e.message, ToastVariant::Error),
                                    }
                                });
                            },
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
