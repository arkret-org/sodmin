//! DTO adapters for the multi-sig partial-signature admin surface.
//!
//! All DTOs come from `soland-contracts`; this module only keeps the UI-facing
//! aliases for the pending-row names.

pub use soland_contracts::admin::seal::{
    MultisigPendingEntry as PendingMultisigSeal, MultisigPendingOutcome, PartialSignatureBody,
    PartialSubmitOutcome, PartialSubmitStatus,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_row_helpers_come_from_shared_type() {
        let row = PendingMultisigSeal {
            seal_id: "ak:seal:1".to_owned(),
            threshold_k: 3,
            threshold_n: 5,
            collected_partials: 1,
            ..Default::default()
        };

        assert_eq!(row.seal_id, "ak:seal:1");
        assert_eq!(row.remaining(), 2);
        assert_eq!(row.threshold_label(), "3 of 5");
        assert!(!row.is_threshold_met());
    }
}
