//! `Table` 系列 — 已迁移到 yoface(`yoface::ui::table::*`)。
//!
//! yoface 版以 `attributes`(`extends=GlobalAttributes`)接收 `class`,
//! `TableCell` 保留 `colspan` 命名 prop,`EmptyRow` 签名一致;调用点
//! `class:` / `colspan:` 透传不变。渲染走 yoface css_module + Soft Orbit 令牌。
pub use yoface::ui::table::{
    EmptyRow, Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
};
