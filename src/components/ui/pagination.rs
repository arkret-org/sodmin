//! `Pagination` / `CursorPagination` — 已迁移到 yoface
//! (`yoface::ui::pagination::*`)。
//!
//! yoface 版签名一致(新增可选 `prev_label` / `next_label`,有默认值),调用点
//! 零改动;内部复用 yoface 的 `Button`,渲染走 css_module + Soft Orbit 令牌。
pub use yoface::ui::pagination::{CursorPagination, Pagination};
