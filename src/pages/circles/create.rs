//! `/circles/new` — create a Circle inside a Realm.
//!
//! Required: parent `realm_id`, `title`. Optional: summary, visibility,
//! join rule, history visibility. The reducer rejects unknown realm
//! refs with `circle_realm_mismatch`; the form surfaces this verbatim
//! when the create call returns.

use dioxus::prelude::*;

use crate::api::circles;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::Spinner;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::types::circles::CreateCircleRequest;
use crate::utils::i18n::t;

#[component]
pub fn CircleCreate() -> Element {
    let mut realm_id = use_signal(String::new);
    let mut title = use_signal(String::new);
    let mut summary = use_signal(String::new);
    let mut directory_visibility = use_signal(|| "members".to_string());
    let mut join_rule = use_signal(|| "invite".to_string());
    let mut history_visibility = use_signal(|| "joined".to_string());
    let mut saving = use_signal(|| false);
    let mut error = use_signal(String::new);
    let nav = use_navigator();

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        if realm_id.read().trim().is_empty() || title.read().trim().is_empty() {
            error.set(t("circle.error_required"));
            return;
        }
        saving.set(true);
        error.set(String::new());
        let req = CreateCircleRequest {
            realm_id: realm_id.read().trim().to_string(),
            title: title.read().trim().to_string(),
            summary: trim_or_none(&summary.read()),
            directory_visibility: trim_or_none(&directory_visibility.read()),
            join_rule: trim_or_none(&join_rule.read()),
            history_visibility: trim_or_none(&history_visibility.read()),
            metadata_encryption_floor: None,
            encryption_profile: None,
        };
        spawn(async move {
            match circles::create_circle(&req).await {
                Ok(c) => {
                    show_toast(&t("circle.created_toast"), ToastVariant::Success);
                    let _ = nav.push(Route::CircleShow { circle_id: c.circle_id });
                }
                Err(e) => {
                    error.set(e.message);
                    saving.set(false);
                }
            }
        });
    };

    let breadcrumbs = vec![
        BreadcrumbItem {
            label: t("circle.list_title"),
            route: Some(Route::CircleList {}),
        },
        BreadcrumbItem {
            label: t("circle.create"),
            route: None,
        },
    ];

    rsx! {
        div { class: "space-y-6",
            Breadcrumbs { items: breadcrumbs }
            PageHeader {
                title: t("circle.create"),
                description: t("circle.create_description"),
            }

            form { class: "space-y-4 max-w-xl", onsubmit: on_submit,
                if !error.read().is_empty() {
                    div { class: "rounded-md bg-destructive/10 p-3 text-sm text-destructive",
                        "{error}"
                    }
                }

                div { class: "space-y-2",
                    Label { r#for: "circle-realm-id".to_string(), {t("circle.realm_id")} }
                    Input {
                        id: "circle-realm-id".to_string(),
                        value: realm_id(),
                        placeholder: "cx:realm:01H...".to_string(),
                        required: true,
                        oninput: move |evt: FormEvent| realm_id.set(evt.value()),
                    }
                    p { class: "text-xs text-muted-foreground",
                        {t("circle.realm_id_hint")}
                    }
                }

                div { class: "space-y-2",
                    Label { r#for: "circle-title".to_string(), {t("circle.title")} }
                    Input {
                        id: "circle-title".to_string(),
                        value: title(),
                        placeholder: t("circle.title_placeholder"),
                        required: true,
                        oninput: move |evt: FormEvent| title.set(evt.value()),
                    }
                }

                div { class: "space-y-2",
                    Label { r#for: "circle-summary".to_string(), {t("circle.summary")} }
                    textarea {
                        id: "circle-summary",
                        name: "summary",
                        class: "flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                        value: "{summary}",
                        oninput: move |e| summary.set(e.value()),
                    }
                }

                div { class: "grid gap-4 md:grid-cols-3",
                    div { class: "space-y-2",
                        Label { r#for: "circle-directory-visibility".to_string(),
                            {t("circle.directory_visibility")}
                        }
                        select {
                            id: "circle-directory-visibility",
                            name: "directory_visibility",
                            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                            onchange: move |e| directory_visibility.set(e.value()),
                            value: directory_visibility(),
                            option { value: "members", "members" }
                            option { value: "realm", "realm" }
                            option { value: "public", "public" }
                        }
                    }
                    div { class: "space-y-2",
                        Label { r#for: "circle-join-rule".to_string(), {t("circle.join_rule")} }
                        select {
                            id: "circle-join-rule",
                            name: "join_rule",
                            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                            onchange: move |e| join_rule.set(e.value()),
                            value: join_rule(),
                            option { value: "invite", "invite" }
                            option { value: "knock", "knock" }
                            option { value: "closed", "closed" }
                        }
                    }
                    div { class: "space-y-2",
                        Label { r#for: "circle-history-visibility".to_string(),
                            {t("circle.history_visibility")}
                        }
                        select {
                            id: "circle-history-visibility",
                            name: "history_visibility",
                            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                            onchange: move |e| history_visibility.set(e.value()),
                            value: history_visibility(),
                            option { value: "joined", "joined" }
                            option { value: "invited", "invited" }
                            option { value: "shared", "shared" }
                            option { value: "world_readable", "world_readable" }
                        }
                    }
                }

                div { class: "flex gap-2 pt-2",
                    Button {
                        r#type: "submit".to_string(),
                        disabled: saving(),
                        if saving() { Spinner { class: "mr-2".to_string() } }
                        {t("common.create")}
                    }
                    Link {
                        to: Route::CircleList {},
                        class: "rounded-md border px-3 py-2 text-sm hover:bg-accent",
                        {t("common.cancel")}
                    }
                }
            }
        }
    }
}

fn trim_or_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::trim_or_none;

    #[test]
    fn trim_or_none_returns_none_for_blank() {
        assert!(trim_or_none("").is_none());
        assert!(trim_or_none("   ").is_none());
        assert_eq!(trim_or_none("  ok  ").as_deref(), Some("ok"));
    }
}
