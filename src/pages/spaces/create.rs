use dioxus::prelude::*;

use crate::api::spaces;
use crate::components::ui::button::Button;
use crate::components::ui::input::Input;
use crate::components::ui::loading::Spinner;
use crate::components::ui::page_header::PageHeader;
use crate::router::Route;
use crate::types::CreateRealmRequest;
use crate::utils::i18n::t;

#[component]
pub fn SpaceCreate() -> Element {
    let mut name = use_signal(String::new);
    let mut topic = use_signal(String::new);
    let mut is_encrypted = use_signal(|| false);
    let discoverability = use_signal(|| "listed".to_string());
    // P3A.6 — required at create time per CXP-0007. Defaults to
    // `collaboration` because principal-control Realms are rare and
    // operators should opt into the heavier classification
    // deliberately.
    let mut realm_class = use_signal(|| "collaboration".to_string());
    let mut saving = use_signal(|| false);
    let mut error = use_signal(String::new);
    let nav = use_navigator();

    let on_submit = move |_evt: Event<FormData>| {
        saving.set(true);
        error.set(String::new());
        let req = CreateRealmRequest {
            title: name.read().clone(),
            topic: if topic.read().is_empty() {
                None
            } else {
                Some(topic.read().clone())
            },
            is_encrypted: is_encrypted(),
            discoverability: if discoverability.read().is_empty() {
                None
            } else {
                Some(discoverability.read().clone())
            },
            // P3A.6 — pin the immutable classification at create time.
            realm_class: realm_class.read().clone(),
            ..Default::default()
        };
        spawn(async move {
            match spaces::create_space(&req).await {
                Ok(space) => {
                    let _ = nav.push(Route::SpaceShow { space_id: space.id });
                }
                Err(e) => {
                    error.set(e.message);
                    saving.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "space-y-6",
            PageHeader { title: t("spaces.create") }

            form { class: "space-y-4 max-w-lg", onsubmit: on_submit,
                if !error.read().is_empty() {
                    div { class: "rounded-md bg-destructive/10 p-3 text-sm text-destructive", "{error}" }
                }

                div { class: "space-y-2",
                    label { class: "text-sm font-medium", {t("spaces.name")} }
                    Input {
                        value: name(),
                        r#type: "text",
                        placeholder: "General",
                        oninput: move |evt: FormEvent| name.set(evt.value()),
                    }
                }

                div { class: "space-y-2",
                    label { class: "text-sm font-medium", {t("spaces.topic")} }
                    textarea {
                        class: "flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                        value: "{topic}",
                        oninput: move |e| topic.set(e.value()),
                    }
                }

                div { class: "flex items-center gap-2",
                    input {
                        r#type: "checkbox",
                        checked: is_encrypted(),
                        onchange: move |e| is_encrypted.set(e.checked()),
                    }
                    label { class: "text-sm", {t("spaces.encrypted")} }
                }

                // P3A.6 — Realm classification picker. Required at
                // create time; immutable afterwards per CXP-0007.
                div { class: "space-y-2",
                    label { class: "text-sm font-medium", {t("realm.classification")} }
                    select {
                        class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                        value: realm_class(),
                        onchange: move |e| realm_class.set(e.value()),
                        option { value: "collaboration", {t("realm.collaboration")} }
                        option { value: "principal_control", {t("realm.principal_control")} }
                    }
                    p { class: "text-xs text-muted-foreground",
                        {t("realm.classification_hint")}
                    }
                }

                div { class: "flex gap-2",
                    Button {
                        r#type: "submit",
                        disabled: saving(),
                        if saving() { Spinner { class: "mr-2".to_string() } }
                        {t("common.create")}
                    }
                    Link {
                        to: Route::SpaceList {},
                        class: "rounded-md border px-3 py-2 text-sm hover:bg-accent",
                        {t("common.cancel")}
                    }
                }
            }
        }
    }
}
