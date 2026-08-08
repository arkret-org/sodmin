pub mod claims_panel;
/// B.7 — shared dangerous-action confirmation dialog with a
/// parameterised typed-phrase gate. Used by /devices revoke (last 4
/// chars of device ID) and /applets/admin suspend / revoke (first 6
/// chars of applet name).
pub mod dangerous_action_dialog;
/// Round 4 — delivery-binding handover panel (`delivery_binding_stale`
/// / `delivery_binding_handed_over` / `historical_only`).
pub mod delivery_binding_handover_panel;
pub mod dev_mode_banner;
pub mod did_binding_panel;
/// Round 4 — DID-shaped input field with inline regex validation
/// (`^did:[a-z0-9]+:[^\s]+$`).
pub mod did_input;
/// R3.2 (UI-SOD-5) — "handle changed since" history hint shown when a
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
/// Generic client-side validating input. Current validation kinds are DID
/// and Required, used by the DID-binding panel `Add binding` form.
pub mod validated_input;
