use dioxus::prelude::*;
// `Label` / `SearchInput` 已迁移到 yoface。
//   * `Label` → `yoface::ui::label::LabelFor`(`r#for` + `class` + children,
//     与本地适配器签名一致,调用点零改动)。
//   * `SearchInput` → `yoface::ui::input::SearchInput`(本地 `label` prop 改名 为
//     `aria_label`;sodmin 现有调用点均未传该 prop,故零改动)。
pub use yoface::ui::input::SearchInput;
pub use yoface::ui::label::LabelFor as Label;

/// `Input` — 已迁移到 yoface(`yoface::ui::input::Input`,css_module 渲染)。
///
/// 本地保留这一薄**适配器**:sodmin 调用点以命名 prop 传 `value` / `placeholder`
/// / `disabled` / `oninput`(非 `Option`)/ 以及一组 HTML 约束(`min_length` /
/// `max_length` / `pattern` / `aria_describedby` 等)。适配器把它们透传为 yoface
/// `Input` 的 attributes(`value` / `placeholder` 等都是 input 全局属性),
/// `oninput` 包成 yoface 的 `Option<EventHandler>`。~未知数量调用点零改动。
#[component]
pub fn Input(
    #[props(default)] id: String,
    #[props(default)] name: String,
    #[props(default = "text".to_string())] r#type: String,
    #[props(default)] class: String,
    #[props(default)] placeholder: String,
    #[props(default)] value: String,
    #[props(default)] aria_label: String,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    /// HTML `minlength` constraint. `None` omits the attribute so
    /// callers that don't care preserve the prior wire shape.
    #[props(default)]
    min_length: Option<u32>,
    /// HTML `maxlength` constraint.
    #[props(default)]
    max_length: Option<u32>,
    /// HTML `pattern` regex for client-side validation. The pattern
    /// is anchored by the browser so callers should not add `^`/`$`.
    #[props(default)]
    pattern: Option<String>,
    /// Optional ARIA description (e.g. inline error message id). When
    /// `pattern` validation fails the browser surfaces this via the
    /// standard `:invalid` pseudoclass; the description hint lets
    /// screen-reader users locate the message.
    #[props(default)]
    aria_describedby: String,
    #[props(default)] oninput: EventHandler<FormEvent>,
) -> Element {
    let resolved_name = if name.is_empty() { id.clone() } else { name };
    rsx! {
        yoface::ui::input::Input {
            id,
            name: resolved_name,
            r#type,
            class: "{class}",
            placeholder,
            value,
            aria_label,
            aria_describedby,
            disabled,
            required,
            minlength: min_length.map(|n| n.to_string()),
            maxlength: max_length.map(|n| n.to_string()),
            pattern: pattern.clone(),
            oninput: move |evt| oninput.call(evt),
        }
    }
}
