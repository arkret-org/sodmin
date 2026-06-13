//! `Card` 系列 — 已迁移到 yoface(`yoface::ui::card::*`)。
//!
//! yoface 版以 `attributes: Vec<Attribute>`(`extends=GlobalAttributes`)接收
//! `class` 等全局属性,调用点 `Card { class: "...", .. }` 透传不变;渲染走
//! yoface css_module + Soft Orbit 令牌。本地不再持有实现。
pub use yoface::ui::card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
