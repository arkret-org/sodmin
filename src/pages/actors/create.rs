use dioxus::prelude::*;

use crate::api::actors;
use crate::components::ui::button::Button;
use crate::components::ui::icons::Icon;
use crate::components::ui::input::Input;
use crate::components::ui::loading::Spinner;
use crate::components::ui::page_header::PageHeader;
use crate::router::Route;
use crate::types::CreateActorRequest;
use crate::utils::handle::{
    HomographReason, is_safe_handle_localpart, normalize_to_canonical,
};
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
        // R3 (UI-5) — refuse to submit a homograph-suspect handle so the
        // operator gets the same answer the soland reducer would have
        // returned (handle_homograph_forbidden). Empty handle is fine —
        // the wire shape allows omitting it entirely.
        //
        // R3.1 (HDLREN-1) — soland's canonical wire form is now
        // `<localpart>:<domain>`. If the operator pasted the display
        // sigil, the `acct:` interop form, or the retired `contrix://`
        // URI, normalise back to canonical before submitting. The
        // homograph guard then runs against the localpart only.
        let raw = handle.read().trim().to_string();
        let normalised_handle: Option<String> = if raw.is_empty() {
            None
        } else if raw.contains(':') || raw.starts_with('@') || raw.starts_with("acct:") {
            match normalize_to_canonical(&raw) {
                Ok(canonical) => {
                    let local = canonical.split(':').next().unwrap_or("");
                    if let Err(reason) = is_safe_handle_localpart(local) {
                        error.set(t(reason.i18n_key()));
                        return;
                    }
                    Some(canonical)
                }
                Err(e) => {
                    error.set(t(e.i18n_key()));
                    return;
                }
            }
        } else {
            // Bare localpart — soland fills in the host domain server-side.
            if let Err(reason) = is_safe_handle_localpart(&raw) {
                error.set(t(reason.i18n_key()));
                return;
            }
            Some(raw)
        };
        saving.set(true);
        error.set(String::new());
        let req = CreateActorRequest {
            handle: normalised_handle,
            display_name: if display_name.read().is_empty() {
                None
            } else {
                Some(display_name.read().clone())
            },
            password: if password.read().is_empty() {
                None
            } else {
                Some(password.read().clone())
            },
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
                    // R3 (UI-5) — inline homograph warning mirroring the
                    // SDK helper `normalize_handle_localpart`. Renders as
                    // soon as the operator types something the soland
                    // reducer would reject with handle_homograph_forbidden.
                    //
                    // R3.1 (HDLREN-1) — if the operator pasted a display
                    // sigil / canonical / acct: form, isolate the
                    // localpart first so the homograph check runs against
                    // only the user-controlled portion. Soland's
                    // canonical wire form is `<localpart>:<domain>`.
                    {
                        let raw = handle.read().trim().to_string();
                        if raw.is_empty() {
                            rsx! {}
                        } else {
                            let local_for_check: String = if raw.contains(':')
                                || raw.starts_with('@')
                                || raw.starts_with("acct:")
                            {
                                match normalize_to_canonical(&raw) {
                                    Ok(canon) => canon
                                        .split(':')
                                        .next()
                                        .unwrap_or("")
                                        .to_string(),
                                    Err(_) => raw.clone(),
                                }
                            } else {
                                raw.clone()
                            };
                            match is_safe_handle_localpart(&local_for_check) {
                                Ok(_) => rsx! {},
                                Err(reason) => {
                                    let copy = t(reason.i18n_key());
                                    let _ = HomographReason::OutOfRange; // touch enum for completeness
                                    rsx! {
                                        p { class: "text-xs text-amber-600 dark:text-amber-300",
                                            span { class: "mr-1", "\u{26A0}" }
                                            "{copy}"
                                        }
                                    }
                                }
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
