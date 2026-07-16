//! The loading/skeleton components have been migrated to yoface
//! (`yoface::ui::loading`, with matching signatures). Re-exported directly, so
//! call sites need zero changes.
pub use yoface::ui::loading::{PageSkeleton, Spinner, StatsSkeleton};
