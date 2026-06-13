//! Checkbox 已迁移到 yoface(`yoface::ui::checkbox`)。
//!
//! sodmin 的列表页表头「全选」与行选择都用三态变体,故本地 `Checkbox`
//! 别名指向 yoface 的 `TristateCheckbox`(API 为本地两态版的超集:多一个
//! `indeterminate`,默认 false)。`header_state` 纯函数同源直接 re-export。
pub use yoface::ui::checkbox::{TristateCheckbox as Checkbox, header_state};
