//! Modal 家族:渲染实现已迁移到 yoface(`yoface::ui::modal`)。本地手写的
//! `fixed inset-0 z-50 …` 遮罩 + 面板实现已删除。
//!
//! 为避免在 ~20 处调用点逐个改 prop 名(其中 `destructive: bool` → `variant`
//! 是类型变更、部分还是动态表达式,盲改易错),这里保留一层**薄适配器**,
//! 把 sodmin 既有 prop 名透传/翻译到 yoface 的新 API:
//!   * `DialogActions.confirm_text`/`cancel_text` → yoface `confirm_label`/`cancel_label`
//!   * `DialogActions.destructive: bool` → yoface `variant: ButtonVariant::{Destructive,Primary}`
//!   * `Modal.max_width: String` → yoface `Modal.class`
//!
//! `ModalOverlay` 签名一致,直接 re-export。
use dioxus::prelude::*;
use yoface::ui::button::ButtonVariant;
pub use yoface::ui::modal::ModalOverlay;

/// 适配器:本地 `max_width: String`(如 `"max-w-lg"`)映射为 yoface `Modal.class`
/// (透传到容器,覆盖默认 max-width)。其余 prop 同名透传。
#[component]
pub fn Modal(
    open: bool,
    #[props(default)] title: String,
    #[props(default = "max-w-md".to_string())] max_width: String,
    on_close: EventHandler<()>,
    children: Element,
) -> Element {
    rsx! {
        yoface::ui::modal::Modal {
            open,
            title,
            class: max_width,
            on_close: move |_| on_close.call(()),
            {children}
        }
    }
}

/// 适配器:`confirm_text`/`cancel_text`/`destructive` → yoface 新 prop 名。
#[component]
pub fn DialogActions(
    confirm_text: String,
    cancel_text: String,
    #[props(default = false)] destructive: bool,
    #[props(default = false)] confirm_loading: bool,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let variant = if destructive {
        ButtonVariant::Destructive
    } else {
        ButtonVariant::Primary
    };
    rsx! {
        yoface::ui::modal::DialogActions {
            confirm_label: confirm_text,
            cancel_label: cancel_text,
            variant,
            confirm_loading,
            on_confirm: move |_| on_confirm.call(()),
            on_cancel: move |_| on_cancel.call(()),
        }
    }
}
