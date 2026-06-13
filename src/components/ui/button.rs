//! `Button` — 已迁移到 yoface(`yoface::ui::button::Button`)。
//!
//! 本地保留这一薄**适配器**:sodmin 调用点用 `ButtonVariant::Default`(yoface
//! 改名为 `Primary`),且习惯以命名 prop 传 `class` / `disabled` / `type` /
//! `onclick`(非 `Option`)。适配器把本地枚举映射到 yoface,并将这些命名 prop
//! 透传为 yoface 的 attributes,渲染走 yoface css_module + Soft Orbit 令牌。
//! 因此 ~140 个调用点零改动。
use dioxus::prelude::*;
use yoface::ui::button::{Button as YButton, ButtonSize as YSize, ButtonVariant as YVariant};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
}

impl ButtonVariant {
    fn to_yoface(self) -> YVariant {
        match self {
            ButtonVariant::Default => YVariant::Primary,
            ButtonVariant::Destructive => YVariant::Destructive,
            ButtonVariant::Outline => YVariant::Outline,
            ButtonVariant::Secondary => YVariant::Secondary,
            ButtonVariant::Ghost => YVariant::Ghost,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
}

impl ButtonSize {
    fn to_yoface(self) -> YSize {
        match self {
            ButtonSize::Default => YSize::Default,
            ButtonSize::Sm => YSize::Sm,
        }
    }
}

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    #[props(default)] class: String,
    #[props(default)] disabled: bool,
    #[props(default = "button".to_string())] r#type: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        YButton {
            variant: variant.to_yoface(),
            size: size.to_yoface(),
            class,
            disabled,
            r#type,
            onclick: move |evt| onclick.call(evt),
            {children}
        }
    }
}
