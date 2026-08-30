pub mod claims_panel;
/// B.7 — shared dangerous-action confirmation dialog with a
/// parameterised typed-phrase gate, used by the destructive mutations
/// on the devices, handles, policy, actors, and coauth admin pages.
pub mod dangerous_action_dialog;
pub mod dev_mode_banner;
pub mod did_binding_panel;
/// DID-shaped input field with inline regex validation
/// (`^did:[a-z0-9]+:[^\s]+$`).
pub mod did_input;
/// "Handle changed since" history hint shown when a
/// captured `handle_at_time` differs from the subject's current primary
/// handle.
pub mod header;
pub mod keyboard_shortcuts;
pub mod layout;
pub mod realm_classification_badge;
pub mod risk_action_panel;
pub mod selection_required;
pub mod sidebar;
pub mod theme;
pub mod ui;
/// Generic client-side validating input for identifiers.
pub mod validated_input;
