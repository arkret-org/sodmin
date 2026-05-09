//! Recovery console + restore-ticket lifecycle admin pages
//! (Round 25, C1-C5).
//!
//! - C1 [`recovery_list`] — paginated list of recovery tickets with
//!   status filter (active by default).
//! - C2 [`recovery_detail`] — per-ticket status timeline + admin
//!   approve/reject/advance/cancel buttons (each in a ConfirmDialog).
//! - C3 [`restore_state`] — per-ticket restore state machine view
//!   (pending → approved → executor_running → complete).
//! - C4 [`recovery_audit`] — audit-log feed for recovery operations.
//! - C5 [`recovery_describe`] — read-only soland recovery describe
//!   surface.

pub mod recovery_audit;
pub mod recovery_describe;
pub mod recovery_detail;
pub mod recovery_list;
pub mod restore_state;
