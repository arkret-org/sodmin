use dioxus::prelude::*;

use crate::api::auth;
use crate::components::ui::loading::Spinner;
use crate::utils::i18n::t;

#[component]
pub fn OAuthCallback(
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
) -> Element {
    let nav = use_navigator();

    use_effect(move || {
        let code = code.clone();
        let state = state.clone();
        let error = error.clone();
        let error_description = error_description.clone();
        spawn(async move {
            if let Some(err) = error.as_ref() {
                log::error!("OAuth error: {} - {:?}", err, error_description);
                nav.replace(crate::router::Route::LoginPage {
                    logout_warning: None,
                });
                return;
            }
            if let Some(code) = code.as_ref() {
                match auth::handle_oauth_callback(code, state.as_deref()).await {
                    Ok(()) => {
                        nav.replace(crate::router::Route::Dashboard {});
                    }
                    Err(e) => {
                        log::error!("OAuth callback failed: {}", e.message);
                        nav.replace(crate::router::Route::LoginPage {
                            logout_warning: None,
                        });
                    }
                }
            } else {
                nav.replace(crate::router::Route::LoginPage {
                    logout_warning: None,
                });
            }
        });
    });

    rsx! {
        div { class: "flex min-h-screen items-center justify-center",
            div { class: "text-center space-y-4",
                Spinner { class: String::new() }
                p { class: "text-muted-foreground", {t("auth.completing_login")} }
            }
        }
    }
}
