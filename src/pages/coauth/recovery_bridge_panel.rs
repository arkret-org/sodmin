use dioxus::prelude::*;

use crate::api::coauth::CoauthRecoveryBridgeDescribe;

pub fn recovery_bridge_block(bridge: &CoauthRecoveryBridgeDescribe) -> Element {
    let verification_kinds = bridge.verification_event_kinds.join(", ");
    let recovery_modes = bridge.recovery_modes.join(", ");
    let backup_example = bridge.example_backup_payload.to_string();
    let restore_example = bridge.recovery_restore_examples.to_string();
    let authz_example = bridge.recovery_authz_examples.to_string();
    let todos_text = bridge.todos.join(" ");

    rsx! {
        h2 { class: "text-base font-semibold", "Recovery Bridge Contract" }
        {path_row("Contract", &bridge.contract)}
        {path_row("Version", &bridge.version)}
        {path_row("Recovery Start", &bridge.recovery_start_path)}
        {path_row("Recovery Status", &bridge.recovery_status_path)}
        {path_row("Recovery Resend", &bridge.recovery_resend_path)}
        {path_row("Coauth Recovery Principal Snapshot", &bridge.recovery_principal_snapshot_path)}
        {path_row("Coauth Recovery Principal Cache Status", &bridge.recovery_principal_cache_status_path)}
        {path_row("Coauth Recovery Principal Cache Refresh", &bridge.recovery_principal_cache_refresh_path)}
        {path_row("Coauth Recovery Principal Cache Queue", &bridge.recovery_principal_cache_queue_path)}
        {path_row("Coauth Recovery Principal Cache Complete", &bridge.recovery_principal_cache_complete_path)}
        {path_row("Coauth Recovery Principal Cache Fail", &bridge.recovery_principal_cache_fail_path)}
        {path_row("Coauth Recovery Principal Cache Policy", &bridge.recovery_principal_cache_policy_path)}
        {path_row("Coauth Recovery Principal Cache Retry", &bridge.recovery_principal_cache_retry_path)}
        {path_row("Coauth Recovery Principal Cache Invalidate", &bridge.recovery_principal_cache_invalidate_path)}
        {path_row("Coauth Recovery Principal Cache Failures", &bridge.recovery_principal_cache_failures_path)}
        {path_row("Coauth Recovery Principal Cache Upstream", &bridge.recovery_principal_cache_upstream_path)}
        {path_row("Coauth Recovery Principal Cache Upstream Probe", &bridge.recovery_principal_cache_upstream_probe_path)}
        {path_row("Coauth Recovery Principal Cache Upstream Bind", &bridge.recovery_principal_cache_upstream_bind_path)}
        {path_row("Key Backup Base", &bridge.key_backup_rest_base)}
        {path_row("Key Backup Schema", &bridge.key_backup_schema)}
        {path_row("Device Message Schema", &bridge.device_message_schema)}
        {path_row("Principal Recovery Contract Stack", &bridge.principal_recovery_contract_stack_path)}
        {path_row("Principal Recovery Stack Bundle", &bridge.principal_recovery_stack_bundle_path)}
        {path_row("Principal Recovery Discovery", &bridge.principal_recovery_discovery_path)}
        {path_row("Principal Recovery Readiness", &bridge.principal_recovery_readiness_path)}
        {path_row("Principal Device Messages Describe", &bridge.principal_device_messages_describe_path)}
        {path_row("Principal Key Backups Describe", &bridge.principal_key_backups_describe_path)}
        {path_row("Principal Restore State Describe", &bridge.principal_restore_state_describe_path)}
        {path_row("Principal Restore State Export", &bridge.principal_restore_state_export_path)}
        {path_row("Principal Restore State Import", &bridge.principal_restore_state_import_path)}
        {path_row("Principal Restore State Durability", &bridge.principal_restore_state_durability_path)}
        {path_row("Principal Restore State Checkpoints", &bridge.principal_restore_state_checkpoint_collection_path)}
        {path_row("Principal Restore Start", &bridge.principal_restore_start_path)}
        {path_row("Principal Restore Describe", &bridge.principal_restore_describe_path)}
        {path_row("Principal Restore Ticket Collection", &bridge.principal_restore_ticket_collection_path)}
        {path_row("Principal Restore Ticket", &bridge.principal_restore_ticket_path)}
        {path_row("Principal Restore Ticket Advance", &bridge.principal_restore_ticket_advance_path)}
        {path_row("Principal Restore Ticket Resume", &bridge.principal_restore_ticket_resume_path)}
        {path_row("Principal Restore Ticket Cancel", &bridge.principal_restore_ticket_cancel_path)}
        {path_row("Principal Restore Ticket Retry", &bridge.principal_restore_ticket_retry_path)}
        {path_row("Principal Restore Approval Status", &bridge.principal_restore_approval_status_path)}
        {path_row("Principal Restore Approval Submit", &bridge.principal_restore_approval_submit_path)}
        {path_row("Principal Restore Executor Status", &bridge.principal_restore_executor_status_path)}
        {path_row("Principal Restore Executor Enqueue", &bridge.principal_restore_executor_enqueue_path)}
        {path_row("Principal Restore Executor Start", &bridge.principal_restore_executor_start_path)}
        {path_row("Principal Restore Executor Complete", &bridge.principal_restore_executor_complete_path)}
        {path_row("Principal Restore Result", &bridge.principal_restore_result_path)}
        {path_row("Principal Restore Receipt", &bridge.principal_restore_receipt_path)}
        {path_row("Principal Restore Materialized Device Handoff", &bridge.principal_restore_materialized_device_handoff_path)}
        {path_row("Principal Restore Bundle", &bridge.principal_restore_bundle_path)}
        {path_row("Principal Restore Activity", &bridge.principal_restore_activity_path)}
        {path_row("Principal Restore Timeline", &bridge.principal_restore_timeline_path)}
        {path_row("Principal Restore Audit Feed", &bridge.principal_restore_audit_feed_path)}
        {path_row("Principal Recovery Live Snapshot", &bridge.principal_recovery_live_snapshot_path)}
        {path_row("Principal Authz Describe", &bridge.principal_authz_describe_path)}
        {path_row("Principal Authz Check", &bridge.principal_authz_check_path)}
        {path_row("Principal Policy Describe", &bridge.principal_policy_describe_path)}
        {path_row("Principal Policy Collection", &bridge.principal_policy_collection_path)}
        {path_row("Principal Policy Item", &bridge.principal_policy_item_path)}
        {path_row("Verification Kinds", &verification_kinds)}
        {path_row("Recovery Modes", &recovery_modes)}
        {path_row("Backup Example", &backup_example)}
        {path_row("Recovery Restore Example", &restore_example)}
        {path_row("Recovery Authz Example", &authz_example)}
        p { class: "text-sm text-muted-foreground", "{todos_text}" }
    }
}

fn path_row(label: &str, value: &str) -> Element {
    rsx! {
        p { class: "text-sm text-muted-foreground",
            "{label}: "
            span { class: "font-mono", "{value}" }
        }
    }
}
