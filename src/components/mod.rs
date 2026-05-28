pub mod claims_panel;
/// B.7 — shared dangerous-action confirmation dialog with a
/// parameterised typed-phrase gate. Used by /devices revoke (last 4
/// chars of device ID) and /applets/admin suspend / revoke (first 6
/// chars of applet name).
pub mod dangerous_action_dialog;
pub mod deactivation_fanout_panel;
/// Round 4 — delivery-binding handover panel (`delivery_binding_stale`
/// / `delivery_binding_handed_over` / `historical_only`).
pub mod delivery_binding_handover_panel;
pub mod dev_mode_banner;
pub mod did_binding_panel;
/// Round 4 — DID-shaped input field with inline regex validation
/// (`^did:[a-z0-9]+:[^\s]+$`).
pub mod did_input;
pub mod footer;
/// P5 — preview of the operator's current capability grants before a
/// destructive action, to avoid the "click → 403 surprise" loop.
pub mod granted_capabilities_view;
/// R3.2 (UI-SOD-5) — "handle changed since" history hint shown when a
/// captured `handle_at_time` differs from the subject's current primary
/// handle.
pub mod handle_change_hint;
pub mod header;
pub mod keyboard_shortcuts;
pub mod layout;
pub mod realm_classification_badge;
pub mod realm_destroy_dialog;
pub mod risk_action_panel;
pub mod sidebar;
pub mod theme;
pub mod ui;
/// P5 — generic client-side validating input (DID / URL / Email /
/// Required / MaxLength). Wired into the personal-agent provision
/// wizard step 1 and the DID-binding panel `Add binding` form.
pub mod validated_input;
