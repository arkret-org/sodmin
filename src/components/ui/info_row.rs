//! `InfoRow` — 已迁移到 yoface(`yoface::ui::info_row::InfoRow`)。
//!
//! sodmin 本地版与 yoface 版签名完全一致(`label` / `value`),故这里改为
//! 直接 re-export yoface 组件:所有 `crate::components::ui::info_row::InfoRow`
//! 调用点零改动,渲染走 yoface css_module + Soft Orbit 令牌。
pub use yoface::ui::info_row::InfoRow;
