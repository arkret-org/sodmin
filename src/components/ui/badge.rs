//! `Badge` — 已迁移到 yoface(`yoface::ui::badge::Badge`)。
//!
//! 本地保留这一薄**适配器**:sodmin 调用点用 `BadgeVariant::Default`(yoface
//! 改名 `Primary`)与 `BadgeVariant::Success`(yoface 无对应语义)。适配器把
//! 可映射的 4 个变体转交 yoface(css_module + Soft Orbit 令牌渲染),`Success`
//! 因 yoface 无该语义,保留本地绿色样式 span 实现。`class` 命名 prop 透传为
//! yoface attributes。~170 个调用点零改动。
use dioxus::prelude::*;
use yoface::ui::badge::{Badge as YBadge, BadgeVariant as YVariant};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Success,
}

#[component]
pub fn Badge(
    #[props(default)] variant: BadgeVariant,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let yv = match variant {
        BadgeVariant::Default => Some(YVariant::Primary),
        BadgeVariant::Secondary => Some(YVariant::Secondary),
        BadgeVariant::Destructive => Some(YVariant::Destructive),
        BadgeVariant::Outline => Some(YVariant::Outline),
        // yoface 无 success 语义,保留本地绿色 span。
        BadgeVariant::Success => None,
    };

    match yv {
        Some(yv) => rsx! {
            YBadge { variant: yv, class, {children} }
        },
        None => rsx! {
            span {
                class: "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold bg-green-500/10 text-green-600 dark:text-green-400 {class}",
                {children}
            }
        },
    }
}
