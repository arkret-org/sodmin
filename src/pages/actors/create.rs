use dioxus::prelude::*;

use crate::api::actors;
use crate::components::ui::button::Button;
use crate::components::ui::card::*;
use crate::components::ui::icons::Icon;
use crate::components::ui::input::Input;
use crate::components::ui::loading::Spinner;
use crate::components::ui::page_header::PageHeader;
use crate::router::Route;
use crate::types::CreateActorRequest;
use crate::utils::i18n::t;

#[component]
pub fn ActorCreate() -> Element {
    let mut handle = use_signal(String::new);
    let mut display_name = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut is_admin = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut handle_check = use_signal(|| Option::<bool>::None);
    let nav = use_navigator();

    let mut check_handle = move || {
        let h = handle.read().clone();
        if h.len() < 3 {
            handle_check.set(None);
            return;
        }
        spawn(async move {
            match actors::check_handle_availability(&h).await {
                Ok(result) => handle_check.set(Some(result.available)),
                Err(_) => handle_check.set(None),
            }
        });
    };

    let on_submit = move |_evt: Event<FormData>| {
        saving.set(true);
        error.set(String::new());
        let req = CreateActorRequest {
            handle: if handle.read().is_empty() { None } else { Some(handle.read().clone()) },
            display_name: if display_name.read().is_empty() { None } else { Some(display_name.read().clone()) },
            password: if password.read().is_empty() { None } else { Some(password.read().clone()) },
            is_admin: is_admin(),
        };
        spawn(async move {
            match actors::create_actor(&req).await {
                Ok(actor) => {
                    let _ = nav.push(Route::ActorShow { actor_id: actor.id });
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
            PageHeader {
                title: t("actors.create"),
            }

            form { class: "space-y-4 max-w-lg", onsubmit: on_submit,
                if !error.read().is_empty() {
                    div { class: "rounded-md bg-destructive/10 p-3 text-sm text-destructive", "{error}" }
                }

                div { class: "space-y-2",
                    label { class: "text-sm font-medium", {t("actors.handle")} }
                    div { class: "flex items-center gap-2",
                        Input {
                            value: handle(),
                            r#type: "text",
                            placeholder: "alice",
                            oninput: move |evt: FormEvent| { handle.set(evt.value()); check_handle(); },
                        }
                        if let Some(available) = handle_check() {
                            if available {
                                Icon { name: "check-circle".to_string(), class: "h-5 w-5 text-green-500".to_string() }
                            } else {
                                Icon { name: "x-circle".to_string(), class: "h-5 w-5 text-red-500".to_string() }
                            }
                        }
                    }
                }

                div { class: "space-y-2",
                    label { class: "text-sm font-medium", {t("actors.display_name")} }
                    Input {
                        value: display_name(),
                        r#type: "text",
                        placeholder: "Alice",
                        oninput: move |evt: FormEvent| display_name.set(evt.value()),
                    }
                }

                div { class: "space-y-2",
                    label { class: "text-sm font-medium", {t("actors.password")} }
                    Input {
                        value: password(),
                        r#type: "password",
                        placeholder: "••••••••",
                        oninput: move |evt: FormEvent| password.set(evt.value()),
                    }
                }

                div { class: "flex items-center gap-2",
                    input {
                        r#type: "checkbox",
                        checked: is_admin(),
                        onchange: move |e| is_admin.set(e.checked()),
                    }
                    label { class: "text-sm", {t("actors.is_admin")} }
                }

                div { class: "flex gap-2",
                    Button {
                        r#type: "submit",
                        disabled: saving(),
                        if saving() { Spinner { class: "mr-2".to_string() } }
                        {t("common.create")}
                    }
                    Link {
                        to: Route::ActorList {},
                        class: "rounded-md border px-3 py-2 text-sm hover:bg-accent",
                        {t("common.cancel")}
                    }
                }
            }
        }
    }
}
