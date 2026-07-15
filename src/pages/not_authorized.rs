use dioxus::prelude::*;

use crate::api::auth;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::router::Route;
use crate::utils::i18n::t;
use crate::utils::net::session;

/// Rendered by `AuthenticatedLayout` when the current user is
/// authenticated but does **not** have homeserver admin privileges.
///
/// Replaces the previous behaviour where every admin page would make its
/// own API call and surface a per-page `M_FORBIDDEN (403)` banner.
#[component]
pub fn NotAuthorizedPage() -> Element {
    let nav = use_navigator();
    let mut logout_in_progress = use_signal(|| false);
    let mut logout_error = use_signal(|| None::<String>);
    let user = session::current_user();
    let display_name = user.display_name.clone().unwrap_or_default();
    let user_id = user.id.clone().unwrap_or_default();

    let handle_logout = move |_evt: MouseEvent| {
        logout_error.set(None);
        logout_in_progress.set(true);
        spawn(async move {
            let report = auth::logout().await;
            if report.local_cookie_cleared() {
                let logout_warning = report.login_warning_code();
                if !report.fully_confirmed() {
                    log::warn!(
                        "local logout succeeded with incomplete upstream cleanup: {report:?}"
                    );
                }
                logout_in_progress.set(false);
                nav.replace(Route::LoginPage { logout_warning });
            } else {
                logout_error.set(Some(report.retry_message()));
                logout_in_progress.set(false);
            }
        });
    };

    rsx! {
        div { class: "flex min-h-screen items-center justify-center bg-background p-4",
            div { class: "w-full max-w-md space-y-6 text-center",
                div { class: "mx-auto h-16 w-16 rounded-full bg-destructive/10 flex items-center justify-center",
                    crate::components::ui::icons::Icon {
                        name: "shield-off".to_string(),
                        class: "h-8 w-8 text-destructive".to_string(),
                    }
                }
                div { class: "space-y-2",
                    h1 { class: "text-2xl font-bold tracking-tight", {t("auth.not_admin")} }
                    p { class: "text-muted-foreground text-sm",
                        "Your account does not have server administrator privileges, so the admin dashboard cannot be shown. Sign out and log in as an administrator, or ask an existing admin to grant you access."
                    }
                }
                if user.has_identity() {
                    div { class: "rounded-lg border glass-panel p-4 text-left space-y-1",
                        p { class: "text-xs uppercase tracking-wide text-muted-foreground",
                            "Signed in as"
                        }
                        if !display_name.is_empty() {
                            p { class: "font-medium", "{display_name}" }
                        }
                        if !user_id.is_empty() {
                            p { class: "text-xs text-muted-foreground break-all", "{user_id}" }
                        }
                    }
                }
                if let Some(message) = logout_error.read().as_ref() {
                    div {
                        class: "rounded-md border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive",
                        role: "alert",
                        "{message}"
                    }
                }
                Button {
                    variant: ButtonVariant::Outline,
                    class: "w-full".to_string(),
                    disabled: *logout_in_progress.read(),
                    onclick: handle_logout,
                    if *logout_in_progress.read() {
                        "Signing out..."
                    } else if logout_error.read().is_some() {
                        "Retry sign-out"
                    } else {
                        {t("auth.sign_out")}
                    }
                }
            }
        }
    }
}
