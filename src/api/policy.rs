use std::collections::BTreeSet;

use serde_json::{Value, json};
use soland_contracts::admin::policy::{
    AdminPolicyDocument, AdminPolicyDocumentPage, AdminPolicyPayload, PolicyEffect,
    UpsertPolicyDocumentRequestBody,
};

use crate::api::client::{NO_BODY, NoBody, api_client, build_url};
use crate::types::policy::{
    AdminPolicy, AdminPolicyListOutcome, CreatePolicyRequest, PinPolicySummary, PolicyAuditEntry,
    PolicyEvidenceItem, PolicyGuardrailSummary, PolicyObligation, PolicyRuleSet,
    PolicySafetySummary,
};
use crate::utils::net::error::HttpError;

pub async fn list_policies(
    _cursor: Option<&str>,
    _limit: u64,
) -> Result<AdminPolicyListOutcome, HttpError> {
    let url = build_url("/_soland/self/policies", &[])?;
    let resp: AdminPolicyDocumentPage = api_client(&url, "GET", NO_BODY).await?;
    Ok(AdminPolicyListOutcome {
        data: resp
            .policies
            .into_iter()
            .map(policy_from_document)
            .collect(),
        total: None,
        next_cursor: resp.next_cursor,
    })
}

pub async fn create_policy(req: &CreatePolicyRequest) -> Result<AdminPolicy, HttpError> {
    if request_targets_pin_policy(req) {
        return Err(pin_policy_unavailable_error());
    }
    let body = upsert_body(None, req)?;
    let resp: AdminPolicyDocument =
        api_client("/_soland/self/policies", "POST", Some(&body)).await?;
    Ok(policy_from_document(resp))
}

pub async fn update_policy(id: &str, req: &CreatePolicyRequest) -> Result<AdminPolicy, HttpError> {
    if request_targets_pin_policy(req) {
        return Err(pin_policy_unavailable_error());
    }
    let body = upsert_body(Some(id.to_string()), req)?;
    let resp: AdminPolicyDocument =
        api_client("/_soland/self/policies", "POST", Some(&body)).await?;
    Ok(policy_from_document(resp))
}

pub async fn delete_policy(id: &str) -> Result<(), HttpError> {
    let url = format!("/_soland/self/policies/{}", urlencoding::encode(id));
    let _: NoBody = api_client(&url, "DELETE", NO_BODY).await?;
    Ok(())
}

fn upsert_body(
    policy_id: Option<String>,
    req: &CreatePolicyRequest,
) -> Result<UpsertPolicyDocumentRequestBody, HttpError> {
    let (effect, actions, resource, obligations) = payload_parts(req)?;
    Ok(UpsertPolicyDocumentRequestBody {
        policy_id,
        scope: req
            .scope
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "*".to_string()),
        subject_ref: req
            .subject_ref
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "*".to_string()),
        policy_kind: req
            .policy_kind
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "*".to_string()),
        effect,
        actions,
        resource,
        obligations,
        active: req.is_enabled,
    })
}

fn payload_parts(
    req: &CreatePolicyRequest,
) -> Result<(PolicyEffect, Vec<String>, Value, Vec<Value>), HttpError> {
    let rules = req.rules.clone().unwrap_or_default();
    let rules_value = rules.as_value();
    let effect = match rules_value.get("effect").and_then(Value::as_str) {
        Some(value) => PolicyEffect::from_wire(value).ok_or_else(|| {
            HttpError::message(format!("unsupported policy effect from server: {value}"))
        })?,
        None => PolicyEffect::Allow,
    };
    let actions = rules_value
        .get("actions")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|values| !values.is_empty())
        .unwrap_or_else(|| vec!["*".to_string()]);
    let obligations = rules_value
        .get("obligations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let resource = json!({
        "name": req.name,
        "description": req.description,
        "priority": req.priority,
        "rules": rules.into_value(),
    });
    Ok((effect, actions, resource, obligations))
}

fn policy_from_document(doc: AdminPolicyDocument) -> AdminPolicy {
    // The guardrail and pin heuristics scan operator-authored JSON, so they
    // stay on `Value`; everything the contract types stays typed.
    let payload = payload_value(&doc.payload);
    let guardrails = policy_guardrails_from_payload(&payload);
    let safety = policy_safety_from_document(&doc);
    let rules = if safety.pin_summary.is_some() {
        Some(PolicyRuleSet::from(pin_policy_public_rules(&safety)))
    } else {
        Some(PolicyRuleSet::from(payload))
    };
    let resource = doc.payload.resource.clone();
    let name = resource
        .get("name")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or(&doc.policy_id)
        .to_string();
    let description = resource
        .get("description")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let priority = resource
        .get("priority")
        .and_then(Value::as_i64)
        .unwrap_or_default() as i32;
    AdminPolicy {
        id: doc.policy_id,
        name,
        policy_kind: Some(doc.policy_kind).filter(|s| !s.is_empty()),
        description,
        scope: Some(doc.scope).filter(|s| !s.is_empty()),
        subject_ref: Some(doc.subject_ref).filter(|s| !s.is_empty()),
        rules,
        guardrails,
        is_enabled: doc.active,
        priority,
        created_at: None,
        updated_at: Some(arkret_canonical::format_timestamp_canonical(doc.updated_at)),
        safety,
    }
}

pub fn request_targets_pin_policy(req: &CreatePolicyRequest) -> bool {
    req.policy_kind
        .as_deref()
        .is_some_and(text_targets_pin_policy)
        || req
            .rules
            .as_ref()
            .is_some_and(|rules| value_targets_pin_policy(rules.as_value()))
}

fn pin_policy_unavailable_error() -> HttpError {
    HttpError::message(
        "pin policy editing is unavailable until soland exposes a standard pin policy summary API",
    )
}

fn payload_value(payload: &AdminPolicyPayload) -> Value {
    serde_json::to_value(payload).unwrap_or(Value::Null)
}

fn policy_safety_from_document(doc: &AdminPolicyDocument) -> PolicySafetySummary {
    if !policy_document_targets_pin(doc) {
        return PolicySafetySummary::default();
    }
    let payload = payload_value(&doc.payload);

    PolicySafetySummary {
        read_only: true,
        read_only_reason: Some(
            "pin policy summary standard surface unavailable; raw private fields are redacted"
                .to_owned(),
        ),
        pin_summary: Some(pin_summary_from_payload(&doc.policy_kind, &payload)),
        redacted_private_categories: private_pin_categories(&payload),
    }
}

fn policy_document_targets_pin(doc: &AdminPolicyDocument) -> bool {
    text_targets_pin_policy(&doc.policy_kind)
        || text_targets_pin_policy(&doc.policy_id)
        || value_targets_pin_policy(&payload_value(&doc.payload))
}

fn text_targets_pin_policy(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    value == "ak.pin"
        || value == "ak.pin.*"
        || value.starts_with("ak.pin.")
        || value.contains(arkret_wire::ProfileId::PINNED_ITEMS_V1)
        || value.contains("pinned_items")
        || value.contains("pin_policy")
        || value.contains("pin.policy")
        || value.contains("pin.quota")
        || value.contains("pin-quota")
        || value.contains("pin_quota")
        || value.contains("pin policy")
}

fn value_targets_pin_policy(value: &Value) -> bool {
    value_targets_pin_policy_in(value, None)
}

fn value_targets_pin_policy_in(value: &Value, context_key: Option<&str>) -> bool {
    match value {
        Value::String(s) => context_key.is_some_and(pin_text_context) && text_targets_pin_policy(s),
        Value::Array(items) => items
            .iter()
            .any(|item| value_targets_pin_policy_in(item, context_key)),
        Value::Object(map) => map.iter().any(|(key, value)| {
            let key_lc = key.to_ascii_lowercase();
            structural_pin_key(&key_lc) || value_targets_pin_policy_in(value, Some(&key_lc))
        }),
        _ => false,
    }
}

fn pin_text_context(key: &str) -> bool {
    matches!(
        key,
        "policy_kind"
            | "type"
            | "kind"
            | "action"
            | "actions"
            | "operation"
            | "operations"
            | "capability"
            | "capabilities"
            | "profile_key"
            | "component"
    )
}

fn structural_pin_key(key: &str) -> bool {
    matches!(
        key,
        "pin_policy"
            | "pin.policy"
            | "pin_scope"
            | "pinned_items"
            | "pin_quota"
            | "pin_limit"
            | "max_pins"
            | "max_pin_count"
            | "max_pinned_items"
            | "max_pins_per_scope"
            | "max_pins_per_realm"
            | "max_pins_per_space"
            | "max_pins_per_circle"
            | "max_pins_per_strand"
    )
}

fn pin_summary_from_payload(policy_kind: &str, payload: &Value) -> PinPolicySummary {
    let mut actions = BTreeSet::new();
    if text_targets_pin_policy(policy_kind) && policy_kind.starts_with("ak.pin.") {
        actions.insert(policy_kind.to_owned());
    }
    collect_pin_actions(payload, &mut actions);

    let mut pin_scopes = BTreeSet::new();
    collect_pin_scopes(payload, &mut pin_scopes);

    let mut quota_limits = BTreeSet::new();
    collect_quota_limits(payload, false, &mut quota_limits);

    PinPolicySummary {
        standard_surface_available: false,
        actions: actions.into_iter().take(12).collect(),
        pin_scopes: pin_scopes.into_iter().take(12).collect(),
        quota_limits: quota_limits.into_iter().take(12).collect(),
        note_plaintext_policy: find_note_plaintext_policy(payload),
    }
}

fn collect_pin_actions(value: &Value, actions: &mut BTreeSet<String>) {
    match value {
        Value::String(s) => {
            if s == "ak.pin.*" || s.starts_with("ak.pin.") {
                actions.insert(s.to_owned());
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_pin_actions(item, actions);
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                collect_pin_actions(value, actions);
            }
        }
        _ => {}
    }
}

fn collect_pin_scopes(value: &Value, pin_scopes: &mut BTreeSet<String>) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_pin_scopes(item, pin_scopes);
            }
        }
        Value::Object(map) => {
            if let Some(scope) = map.get("pin_scope").and_then(Value::as_object) {
                let kind = scope
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let id = scope.get("id").and_then(Value::as_str).unwrap_or("unknown");
                pin_scopes.insert(format!("{}:{}", safe_label(kind), safe_label(id)));
            }
            for value in map.values() {
                collect_pin_scopes(value, pin_scopes);
            }
        }
        _ => {}
    }
}

fn collect_quota_limits(value: &Value, quota_context: bool, limits: &mut BTreeSet<String>) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_quota_limits(item, quota_context, limits);
            }
        }
        Value::Object(map) => {
            for (key, value) in map {
                let key_lc = key.to_ascii_lowercase();
                let next_quota_context = quota_context || key_lc.contains("quota");
                if is_public_quota_key(&key_lc) {
                    if let Some(label) = public_scalar_label(value) {
                        limits.insert(format!("{key}={label}"));
                    }
                } else if next_quota_context
                    && is_public_quota_leaf(&key_lc)
                    && let Some(label) = public_scalar_label(value)
                {
                    limits.insert(format!("{key}={label}"));
                }
                collect_quota_limits(value, next_quota_context, limits);
            }
        }
        _ => {}
    }
}

fn is_public_quota_key(key: &str) -> bool {
    matches!(
        key,
        "max_pins"
            | "max_pin_count"
            | "max_pinned_items"
            | "max_pins_per_scope"
            | "max_pins_per_realm"
            | "max_pins_per_space"
            | "max_pins_per_circle"
            | "max_pins_per_strand"
            | "pin_limit"
            | "pin_quota"
    )
}

fn is_public_quota_leaf(key: &str) -> bool {
    matches!(
        key,
        "max" | "limit" | "max_operations" | "max_items" | "period" | "window" | "enabled"
    )
}

fn public_scalar_label(value: &Value) -> Option<String> {
    match value {
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(v) => Some(v.to_string()),
        Value::String(s) if is_safe_scalar_string(s) => Some(s.to_owned()),
        _ => None,
    }
}

fn is_safe_scalar_string(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ':' | '/' | ' '))
}

fn find_note_plaintext_policy(value: &Value) -> Option<String> {
    match value {
        Value::Array(items) => items.iter().find_map(find_note_plaintext_policy),
        Value::Object(map) => {
            for (key, value) in map {
                let key_lc = key.to_ascii_lowercase();
                if matches!(
                    key_lc.as_str(),
                    "note_plaintext_policy"
                        | "pin_note_plaintext"
                        | "plaintext_notes"
                        | "note_visibility"
                ) && let Some(label) = public_scalar_label(value)
                {
                    return Some(label);
                }
                if let Some(found) = find_note_plaintext_policy(value) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

fn private_pin_categories(value: &Value) -> Vec<String> {
    let mut categories = BTreeSet::new();
    collect_private_pin_categories(value, &mut categories);
    categories.into_iter().collect()
}

fn collect_private_pin_categories(value: &Value, categories: &mut BTreeSet<String>) {
    match value {
        Value::String(s) => add_private_category_for_text(s, categories),
        Value::Array(items) => {
            for item in items {
                collect_private_pin_categories(item, categories);
            }
        }
        Value::Object(map) => {
            for (key, value) in map {
                add_private_category_for_text(key, categories);
                collect_private_pin_categories(value, categories);
            }
        }
        _ => {}
    }
}

fn add_private_category_for_text(value: &str, categories: &mut BTreeSet<String>) {
    let normalized = value.to_ascii_lowercase().replace('-', "_");
    if normalized.contains("account_data")
        || normalized.contains("accountdata")
        || normalized.contains(arkret_wire::AccountDataKey::SEARCH_INDEX_MANIFEST_V1)
        || normalized.contains(arkret_wire::AccountDataKey::SAVED_V1)
        || normalized.contains(arkret_wire::AccountDataKey::REMINDERS_V1)
    {
        categories.insert("private account-data".to_owned());
    }
    if normalized.contains("search_index")
        || normalized.contains("index_manifest")
        || normalized.contains("search.index")
    {
        categories.insert("search index".to_owned());
    }
    if normalized.contains("blind_token")
        || normalized.contains("search_token")
        || normalized.contains("shard_key")
        || normalized.contains("shard_token")
    {
        categories.insert("search token/shard".to_owned());
    }
}

fn safe_label(value: &str) -> String {
    if value.len() <= 128 {
        return value.to_owned();
    }
    let truncated: String = value.chars().take(128).collect();
    format!("{truncated}...")
}

fn pin_policy_public_rules(safety: &PolicySafetySummary) -> Value {
    let Some(summary) = safety.pin_summary.as_ref() else {
        return json!({});
    };
    json!({
        "kind": "pin_policy_admin_summary",
        "standard_surface_available": summary.standard_surface_available,
        "actions": &summary.actions,
        "pin_scopes": &summary.pin_scopes,
        "quota_limits": &summary.quota_limits,
        "note_plaintext_policy": &summary.note_plaintext_policy,
        "redacted_private_categories": &safety.redacted_private_categories,
    })
}

fn policy_guardrails_from_payload(payload: &Value) -> PolicyGuardrailSummary {
    let mut guardrails = PolicyGuardrailSummary {
        required_scope: string_field(payload, &["required_scope"])
            .or_else(|| nested_string_field(payload, "authorization", &["required_scope"]))
            .or_else(|| nested_string_field(payload, "resource", &["required_scope"])),
        approval_evidence: Vec::new(),
        audit_trail: Vec::new(),
        obligations: payload
            .get("obligations")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .cloned()
                    .map(PolicyObligation::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    };

    guardrails
        .approval_evidence
        .extend(evidence_items(payload, "approval_evidence"));
    if let Some(resource) = payload.get("resource") {
        guardrails
            .approval_evidence
            .extend(evidence_items(resource, "approval_evidence"));
    }

    guardrails
        .audit_trail
        .extend(audit_entries(payload, "audit_trail"));
    if let Some(resource) = payload.get("resource") {
        guardrails
            .audit_trail
            .extend(audit_entries(resource, "audit_trail"));
    }

    guardrails
}

fn evidence_items(value: &Value, key: &str) -> Vec<PolicyEvidenceItem> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(evidence_item)
        .collect()
}

fn evidence_item(value: &Value) -> Option<PolicyEvidenceItem> {
    let obj = value.as_object()?;
    let field = |keys: &[&str]| {
        keys.iter()
            .find_map(|key| obj.get(*key).and_then(Value::as_str))
            .map(str::to_owned)
            .filter(|value| !value.trim().is_empty())
    };
    Some(PolicyEvidenceItem {
        kind: field(&["kind", "type", "proof_kind"])?,
        reference: field(&["evidence_ref", "id", "approval_id", "request_id", "digest"])?,
        actor: field(&["approved_by", "issuer", "actor_id", "subject_id"]),
        decision: field(&["decision", "outcome", "state"]),
        digest: field(&["digest", "proof_digest", "request_canonical_digest"]),
        issued_at: field(&["approved_at", "issued_at", "timestamp"]),
    })
}

fn audit_entries(value: &Value, key: &str) -> Vec<PolicyAuditEntry> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(audit_entry)
        .collect()
}

fn audit_entry(value: &Value) -> Option<PolicyAuditEntry> {
    let obj = value.as_object()?;
    let field = |keys: &[&str]| {
        keys.iter()
            .find_map(|key| obj.get(*key).and_then(Value::as_str))
            .map(str::to_owned)
            .filter(|value| !value.trim().is_empty())
    };
    Some(PolicyAuditEntry {
        action: field(&["action", "operation", "kind"])?,
        actor: field(&["actor", "actor_id", "admin", "principal_id"]),
        outcome: field(&["outcome", "decision", "state"]),
        request_id: field(&["request_id", "trace_id"]),
        timestamp: field(&["timestamp", "created_at", "at"]),
    })
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::to_owned)
        .filter(|value| !value.trim().is_empty())
}

fn nested_string_field(value: &Value, parent: &str, keys: &[&str]) -> Option<String> {
    value
        .get(parent)
        .and_then(|child| string_field(child, keys))
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use serde_json::json;

    use super::*;

    fn fixture_timestamp() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-08-14T00:00:00Z")
            .expect("fixture timestamp")
            .with_timezone(&Utc)
    }

    #[test]
    fn policy_guardrails_extract_approval_evidence_and_audit_trail() {
        let payload = json!({
            "effect": "require_review",
            "actions": ["ak.realm.policy.update"],
            "resource": {
                "required_scope": "realm.admin.write",
                "approval_evidence": [{
                    "kind": "human_approval",
                    "evidence_ref": "ak:event:Aa-XDJ66vTapNhYhwHWSDwNsiA2MQQQ7lLx-iCzgfgaH",
                    "approved_by": "did:web:admin.example",
                    "decision": "approved",
                    "request_canonical_digest": "sha256:abc",
                    "approved_at": "2026-06-19T00:00:00Z"
                }],
                "audit_trail": [{
                    "action": "policy.update",
                    "actor_id": "did:web:admin.example",
                    "outcome": "accepted",
                    "request_id": "req_1",
                    "timestamp": "2026-06-19T00:00:01Z"
                }]
            },
            "obligations": [{"kind": "approval_required"}]
        });

        let guardrails = policy_guardrails_from_payload(&payload);
        assert_eq!(
            guardrails.required_scope.as_deref(),
            Some("realm.admin.write")
        );
        assert_eq!(guardrails.approval_evidence.len(), 1);
        assert_eq!(guardrails.approval_evidence[0].kind, "human_approval");
        assert_eq!(
            guardrails.approval_evidence[0].actor.as_deref(),
            Some("did:web:admin.example")
        );
        assert_eq!(guardrails.audit_trail.len(), 1);
        assert_eq!(guardrails.audit_trail[0].action, "policy.update");
        assert_eq!(guardrails.obligations.len(), 1);
    }

    #[test]
    fn policy_from_document_carries_guardrail_summary() {
        let doc = AdminPolicyDocument {
            policy_id: "ak:policy:9f18274d-cb85-75eb-9891-eff82acff4c0".to_owned(),
            owner: "did:web:admin.example".to_owned(),
            scope: "ak:realm:ASrEhTMQSRXph9wD98UfShBx6MNM7ISQiBIDT0Ayiozq".to_owned(),
            subject_ref: "did:web:admin.example".to_owned(),
            policy_kind: "ak.realm.policy.update".to_owned(),
            payload: AdminPolicyPayload {
                effect: PolicyEffect::Allow,
                actions: Vec::new(),
                resource: json!({
                    "name": "Realm policy",
                    "priority": 3,
                    "audit_trail": [{"action": "policy.create"}]
                }),
                obligations: Vec::new(),
            },
            active: true,
            updated_at: fixture_timestamp(),
        };

        let policy = policy_from_document(doc);
        assert_eq!(policy.name, "Realm policy");
        assert_eq!(policy.priority, 3);
        assert_eq!(policy.subject_ref.as_deref(), Some("did:web:admin.example"));
        assert_eq!(policy.guardrails.audit_trail.len(), 1);
    }

    #[test]
    fn upsert_body_preserves_subject_ref_and_hard_deny_effect() {
        let req = CreatePolicyRequest {
            name: "Targeted deny".to_owned(),
            scope: Some("ak:realm:AcTCbPKkRYcSVLkuEFQjnPuFXfCsFvSRZDAjCsho3_b-".to_owned()),
            subject_ref: Some("did:web:bob.example".to_owned()),
            policy_kind: Some("ak.message.send".to_owned()),
            rules: Some(
                json!({
                    "effect": "hard_deny",
                    "actions": ["ak.message.send"]
                })
                .into(),
            ),
            is_enabled: true,
            ..Default::default()
        };

        let body = upsert_body(
            Some("ak:policy:f54a950e-b8e5-70af-92bd-4f27a495b754".to_owned()),
            &req,
        )
        .unwrap();
        let encoded = serde_json::to_value(body).unwrap();

        assert_eq!(encoded["subject_ref"], "did:web:bob.example");
        assert_eq!(encoded["effect"], "hard_deny");
        assert_eq!(encoded["actions"], json!(["ak.message.send"]));
    }

    #[test]
    fn upsert_body_rejects_unknown_policy_effect() {
        let req = CreatePolicyRequest {
            name: "Bad effect".to_owned(),
            rules: Some(json!({"effect": "deny"}).into()),
            ..Default::default()
        };

        assert!(upsert_body(None, &req).is_err());
    }

    #[test]
    fn pin_policy_document_is_read_only_and_redacts_private_material() {
        let doc = AdminPolicyDocument {
            policy_id: "ak:policy:577e618f-3aed-75d9-9cb2-8fc1199b163a".to_owned(),
            owner: "did:web:admin.example".to_owned(),
            scope: "ak:realm:AcjPhXNC5Rr73hdU3Lep53Z0K69AczAn915GJwCtexkF".to_owned(),
            subject_ref: "*".to_owned(),
            policy_kind: "ak.profile.pinned_items.v1".to_owned(),
            payload: AdminPolicyPayload {
                effect: PolicyEffect::Allow,
                actions: vec!["ak.pin.add".to_owned(), "ak.pin.reorder".to_owned()],
                resource: json!({
                    "name": "Realm pins",
                    "pin_scope": {"kind": "realm", "id": "ak:realm:AcjPhXNC5Rr73hdU3Lep53Z0K69AczAn915GJwCtexkF"},
                    "quota": {"max_pins_per_scope": 5, "period": "PT1H"},
                    "note_visibility": "encrypted",
                    "account_data_key": "ak.search.index_manifest.v1:secret-realm-key",
                    "search_index": {"shard_key": "super-secret-token"}
                }),
                obligations: Vec::new(),
            },
            active: true,
            updated_at: fixture_timestamp(),
        };

        let policy = policy_from_document(doc);
        assert!(policy.safety.read_only);
        assert_eq!(
            policy.safety.redacted_private_categories,
            vec![
                "private account-data".to_owned(),
                "search index".to_owned(),
                "search token/shard".to_owned()
            ]
        );

        let summary = policy.safety.pin_summary.as_ref().unwrap();
        assert!(!summary.standard_surface_available);
        assert!(summary.actions.iter().any(|v| v == "ak.pin.add"));
        assert!(
            summary
                .quota_limits
                .iter()
                .any(|v| v == "max_pins_per_scope=5")
        );
        assert_eq!(summary.note_plaintext_policy.as_deref(), Some("encrypted"));

        let public_rules = serde_json::to_string(&policy.rules.unwrap()).unwrap();
        assert!(!public_rules.contains("secret-realm-key"));
        assert!(!public_rules.contains("super-secret-token"));
        assert!(public_rules.contains("private account-data"));
        assert!(public_rules.contains("search index"));
    }

    #[test]
    fn pin_policy_mutation_requests_fail_closed_before_network() {
        let pin_by_type = CreatePolicyRequest {
            policy_kind: Some("ak.pin.add".to_owned()),
            ..Default::default()
        };
        assert!(request_targets_pin_policy(&pin_by_type));

        let pin_by_rules = CreatePolicyRequest {
            rules: Some(json!({"actions": ["ak.pin.reorder"]}).into()),
            ..Default::default()
        };
        assert!(request_targets_pin_policy(&pin_by_rules));

        let ordinary = CreatePolicyRequest {
            policy_kind: Some("ak.message.create".to_owned()),
            rules: Some(json!({"actions": ["ak.message.create"]}).into()),
            ..Default::default()
        };
        assert!(!request_targets_pin_policy(&ordinary));

        let ordinary_with_free_text = CreatePolicyRequest {
            policy_kind: Some("ak.message.create".to_owned()),
            rules: Some(
                json!({"resource": {"description": "mentions pin policy in prose"}}).into(),
            ),
            ..Default::default()
        };
        assert!(!request_targets_pin_policy(&ordinary_with_free_text));
    }
}
