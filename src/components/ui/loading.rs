//! 加载/骨架屏组件已迁移到 yoface(`yoface::ui::loading`,签名一致)。
//! 直接 re-export,调用点零改动。
pub use yoface::ui::loading::{PageSkeleton, Spinner, StatsSkeleton};
