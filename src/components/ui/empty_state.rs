//! `EmptyState` 已迁移到 yoface(`yoface::ui::empty_state::EmptyState`)。
//!
//! 调用点改名(随迁移一并处理):`icon: "name".to_string()` →
//! `icon_name: "name".to_string()`(yoface 用 lucide kebab-case 名映射内置图标,
//! 不再维护本地字符串图标注册表)。动作仍是 `action_label` + `action_href`
//! (`<a href>`),sodmin 现有调用点均未使用。
pub use yoface::ui::empty_state::EmptyState;
