pub use cokret_contracts::ops::HardeningStatus;
pub use cokret_core::model::{
    DirectoryListHandlesForSubjectReqBody as ListHandlesForSubjectRequest,
    DirectoryListHandlesForSubjectResBody as ListHandlesForSubjectResponse, HandleBindingState,
    HandleClaim, MemberDeliveryBinding,
};
use serde::{Deserialize, Serialize};

// ── Pagination ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub total: u64,
    /// Opaque cursor for the next page. `None` when the current page is
    /// the last one.
    #[serde(default)]
    pub next_cursor: Option<String>,
}
