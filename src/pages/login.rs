use dioxus::prelude::*;

use crate::api::auth;
use crate::components::ui::button::Button;
use crate::components::ui::loading::Spinner;
use crate::utils::i18n::t;

#[component]
pub fn LoginPage() -> Element {
    let mut loading = use_signal(|| false);
    let mut ready = use_signal(|| false);
    let mut config_error = use_signal::<Option<String>>(|| None);

    use_effect(move || {
        spawn(async move {
            match crate::utils::net::config::load_runtime_config().await {
                Ok(cfg) => {
                    crate::utils::storage::set_item("coauth_public_url", &cfg.coauth_public_url);
                    // P5 — propagate the optional telemetry endpoint
                    // into localStorage so `utils::net::telemetry` can pick
                    // it up the first time a report fires.
                    crate::utils::net::telemetry::set_endpoint(&cfg.telemetry_endpoint);
                    config_error.set(None);
                    ready.set(true);
                }
                Err(err) => {
                    config_error.set(Some(err.to_string()));
                    ready.set(true);
                }
            }
        });
    });

    let handle_login = move |_evt: MouseEvent| {
        loading.set(true);
        spawn(async move {
            auth::start_oauth_login().await;
            loading.set(false);
        });
    };

    let is_ready = *ready.read();
    let is_loading = *loading.read();
    let error_message = config_error.read().clone();

    if let Some(message) = error_message {
        return rsx! { ConfigErrorPanel { message } };
    }

    rsx! {
        div { class: "flex min-h-screen items-center justify-center bg-background p-4", role: "main",
            div { class: "w-full max-w-md space-y-6",
                div { class: "text-center space-y-2",
                    div { class: "mx-auto h-16 w-16 rounded-full bg-primary/10 flex items-center justify-center",
                        crate::components::ui::icons::Icon {
                            name: "shield".to_string(),
                            class: "h-8 w-8 text-primary".to_string(),
                        }
                    }
                    h1 { class: "text-2xl font-bold tracking-tight", "Cokret Admin" }
                    p { class: "text-muted-foreground", {t("auth.sign_in_subtitle")} }
                }

                div { class: "rounded-lg border glass-panel p-6 shadow-sm space-y-4",
                    p { class: "text-sm text-center text-muted-foreground",
                        {t("auth.oauth_hint")}
                    }

                    Button {
                        class: "w-full".to_string(),
                        disabled: !is_ready || is_loading,
                        onclick: handle_login,
                        if is_loading {
                            Spinner { class: "mr-2".to_string() }
                        }
                        {t("auth.sign_in")}
                    }
                }

                p { class: "text-center text-xs text-muted-foreground",
                    {t("auth.footer")}
                }
            }
        }
    }
}

/// Hard error surface shown when `/config.json` is unreachable or missing
/// `coauth_public_url`. Sign-in is impossible without this URL — the
/// OAuth redirect would point to a relative path and fail — so we
/// short-circuit with a deployment-ops-facing message instead of a dead
/// sign-in button.
#[component]
fn ConfigErrorPanel(message: String) -> Element {
    rsx! {
        div { class: "flex min-h-screen items-center justify-center bg-background p-4", role: "main",
            div { class: "w-full max-w-lg space-y-6",
                div { class: "text-center space-y-2",
                    div { class: "mx-auto h-16 w-16 rounded-full bg-destructive/10 flex items-center justify-center",
                        crate::components::ui::icons::Icon {
                            name: "alert-triangle".to_string(),
                            class: "h-8 w-8 text-destructive".to_string(),
                        }
                    }
                    h1 { class: "text-2xl font-bold tracking-tight",
                        {t("config.error.title")}
                    }
                    p { class: "text-muted-foreground",
                        {t("config.error.subtitle")}
                    }
                }

                div { class: "rounded-lg border border-destructive/40 bg-destructive/5 p-4 text-sm",
                    p { class: "font-mono whitespace-pre-wrap break-all", "{message}" }
                }

                div { class: "rounded-lg border glass-panel p-4 text-sm space-y-2",
                    p { class: "font-medium", {t("config.error.hint_title")} }
                    ul { class: "list-disc list-inside text-muted-foreground space-y-1",
                        li { {t("config.error.hint_serve_path")} }
                        li { {t("config.error.hint_required_fields")} }
                        li { {t("config.error.hint_check_proxy")} }
                    }
                }
            }
        }
    }
}
