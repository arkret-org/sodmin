use dioxus::prelude::*;

use super::button::{Button, ButtonVariant};

/// Full-screen dim backdrop + centered container used by every hand-rolled
/// `fixed inset-0 z-50 ...` dialog across the list/detail pages.
///
/// Renders nothing when `open` is false. Clicking the backdrop fires
/// `on_close`. `title` renders the standard `h2` header; pass form fields
/// and a [`DialogActions`] row as `children`. `max_width` overrides the
/// default `max-w-md` container width (e.g. `"max-w-lg"`).
#[component]
pub fn Modal(
    open: bool,
    #[props(default)] title: String,
    #[props(default = "max-w-md".to_string())] max_width: String,
    on_close: EventHandler<()>,
    children: Element,
) -> Element {
    if !open {
        return rsx! {};
    }

    rsx! {
        ModalOverlay { on_close,
            div { class: "relative z-50 w-full {max_width} rounded-lg border glass-panel p-6 shadow-lg space-y-4",
                if !title.is_empty() {
                    h2 { class: "text-lg font-semibold", "{title}" }
                }
                {children}
            }
        }
    }
}

/// Bare centered overlay (backdrop + flex container) without the standard
/// dialog panel. Use when a page needs a custom panel shell but still wants
/// the shared backdrop + click-to-close behavior.
#[component]
pub fn ModalOverlay(on_close: EventHandler<()>, children: Element) -> Element {
    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center",
            div {
                class: "fixed inset-0 bg-black/80",
                onclick: move |_| on_close.call(()),
            }
            {children}
        }
    }
}

/// Trailing confirm/cancel button row shared by every dialog
/// (`flex justify-end gap-2` + Outline cancel + primary/destructive confirm).
///
/// `confirm_loading` disables the confirm button while a mutation is in
/// flight. Set `destructive` to render the confirm button in the
/// destructive variant.
#[component]
pub fn DialogActions(
    confirm_text: String,
    cancel_text: String,
    #[props(default = false)] destructive: bool,
    #[props(default = false)] confirm_loading: bool,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "flex justify-end gap-2",
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| on_cancel.call(()),
                "{cancel_text}"
            }
            Button {
                variant: if destructive { ButtonVariant::Destructive } else { ButtonVariant::Default },
                disabled: confirm_loading,
                onclick: move |_| on_confirm.call(()),
                "{confirm_text}"
            }
        }
    }
}
