pub mod claims_panel;
pub mod deactivation_fanout_panel;
/// Round 4 — delivery-binding handover panel (`delivery_binding_stale`
/// / `delivery_binding_handed_over` / `historical_only`).
pub mod delivery_binding_handover_panel;
pub mod dev_mode_banner;
/// Round 4 — DID-shaped input field with inline regex validation
/// (`^did:[a-z0-9]+:[^\s]+$`).
pub mod did_input;
pub mod did_binding_panel;
pub mod footer;
pub mod header;
pub mod keyboard_shortcuts;
pub mod layout;
pub mod realm_destroy_dialog;
pub mod risk_action_panel;
pub mod sidebar;
pub mod theme;
pub mod ui;
