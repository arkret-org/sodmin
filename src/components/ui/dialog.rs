//! `ConfirmDialog`:渲染实现已迁移到 yoface(`yoface::ui::modal::ConfirmDialog`)。
//! 本地手写的遮罩 + 面板实现已删除。
//!
//! 保留一层**薄适配器**,把 sodmin 既有 prop 名透传/翻译到 yoface 新 API:
//!   * `description` → yoface `message`
//!   * `confirm_text`/`cancel_text` → yoface `confirm_label`/`cancel_label`
//!   * `destructive: bool` → yoface `variant: ButtonVariant::{Destructive,Primary}`
//! 避免在 ~20 处调用点逐个改 prop 名(`destructive` 含动态表达式,盲改易错)。
use dioxus::prelude::*;
use yoface::ui::button::ButtonVariant;

#[component]
pub fn ConfirmDialog(
    open: bool,
    title: String,
    description: String,
    #[props(default = "Confirm".to_string())] confirm_text: String,
    #[props(default = "Cancel".to_string())] cancel_text: String,
    #[props(default = false)] destructive: bool,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let variant = if destructive {
        ButtonVariant::Destructive
    } else {
        ButtonVariant::Primary
    };
    rsx! {
        yoface::ui::modal::ConfirmDialog {
            open,
            title,
            message: description,
            confirm_label: confirm_text,
            cancel_label: cancel_text,
            variant,
            on_confirm: move |_| on_confirm.call(()),
            on_cancel: move |_| on_cancel.call(()),
        }
    }
}
