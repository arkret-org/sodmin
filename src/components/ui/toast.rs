//! Toast — 已迁移到 yoface(`yoface::ui::toast::*`)。
//!
//! 直接 re-export:`Toaster` / `show_toast` / `ToastVariant` 等调用点零改动,
//! 且 `show_toast` 与 `Toaster` 共享 yoface 内部同一 `TOASTS` GlobalSignal,
//! 渲染走 yoface css_module + Soft Orbit 令牌。
pub use yoface::ui::toast::{
    TOASTS, Toast, ToastAction, ToastVariant, Toaster, show_toast, show_toast_with_action,
};
