use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::api::client::{api_client, build_url, json_body};
use crate::api::contracts::soland_admin::{CreatePolicyRequest, Policy, PolicyListOutcome};
use crate::types::policy::{PolicyAuditEntry, PolicyEvidenceItem, PolicyGuardrailSummary};
use crate::utils::net::error::HttpError;

#[derive(Debug, Clone, Deserialize, Default)]
struct PolicyDocumentsEnvelope {
    #[serde(default)]
    policies: Vec<PolicyDocumentDto>,
    #[serde(default)]
    next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PolicyDocumentDto {
    #[serde(default)]
    policy_id: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    policy_type: String,
    #[serde(default)]
    payload: Value,
    #[serde(default)]
    active: bool,
    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct UpsertPolicyDocumentRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    policy_id: Option<String>,
    scope: String,
    subject_ref: String,
    policy_type: String,
    effect: String,
    actions: Vec<String>,
    resource: Value,
    obligations: Vec<Value>,
    active: bool,
}

pub async fn list_policies(
    cursor: Option<&str>,
    limit: u64,
) -> Result<PolicyListOutcome, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str())];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url("/_soland/self/policies", &params)?;
    let resp: PolicyDocumentsEnvelope = api_client(&url, "GET", None).await?;
    Ok(PolicyListOutcome {
        data: resp
            .policies
            .into_iter()
            .map(policy_from_document)
            .collect(),
        total: None,
        next_cursor: resp.next_cursor,
    })
}

pub async fn create_policy(req: &CreatePolicyRequest) -> Result<Policy, HttpError> {
    let body = upsert_body(None, req);
    let resp: PolicyDocumentDto =
        api_client("/_soland/self/policies", "POST", Some(json_body(&body)?)).await?;
    Ok(policy_from_document(resp))
}

pub async fn update_policy(id: &str, req: &CreatePolicyRequest) -> Result<Policy, HttpError> {
    let body = upsert_body(Some(id.to_string()), req);
    let resp: PolicyDocumentDto =
        api_client("/_soland/self/policies", "POST", Some(json_body(&body)?)).await?;
    Ok(policy_from_document(resp))
}

pub async fn delete_policy(id: &str) -> Result<(), HttpError> {
    let url = format!("/_soland/self/policies/{}", urlencoding::encode(id));
    let _: serde_json::Value = api_client(&url, "DELETE", None).await?;
    Ok(())
}

fn upsert_body(
    policy_id: Option<String>,
    req: &CreatePolicyRequest,
) -> UpsertPolicyDocumentRequestBody {
    let (effect, actions, resource, obligations) = payload_parts(req);
    UpsertPolicyDocumentRequestBody {
        policy_id,
        scope: req
            .scope
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "*".to_string()),
        subject_ref: "*".to_string(),
        policy_type: req
            .policy_type
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "*".to_string()),
        effect,
        actions,
        resource,
        obligations,
        active: req.is_enabled,
    }
}

fn payload_parts(req: &CreatePolicyRequest) -> (String, Vec<String>, Value, Vec<Value>) {
    let rules = req.rules.clone().unwrap_or_else(|| json!({}));
    let effect = rules
        .get("effect")
        .and_then(Value::as_str)
        .unwrap_or("allow")
        .to_string();
    let actions = rules
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
    let obligations = rules
        .get("obligations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let resource = json!({
        "name": req.name,
        "description": req.description,
        "priority": req.priority,
        "rules": rules,
    });
    (effect, actions, resource, obligations)
}

fn policy_from_document(doc: PolicyDocumentDto) -> Policy {
    let guardrails = policy_guardrails_from_payload(&doc.payload);
    let resource = doc.payload.get("resource").cloned().unwrap_or(Value::Null);
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
    Policy {
        id: doc.policy_id,
        name,
        policy_type: Some(doc.policy_type).filter(|s| !s.is_empty()),
        description,
        scope: Some(doc.scope).filter(|s| !s.is_empty()),
        rules: Some(doc.payload),
        guardrails,
        is_enabled: doc.active,
        priority,
        created_at: None,
        updated_at: doc.updated_at,
    }
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
            .cloned()
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
        kind: field(&["kind", "type", "proof_kind"]).unwrap_or_else(|| "approval".to_owned()),
        reference: field(&[
            "evidence_ref",
            "ref",
            "id",
            "approval_id",
            "request_id",
            "digest",
        ])
        .unwrap_or_else(|| "inline".to_owned()),
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
        action: field(&["action", "operation", "kind"]).unwrap_or_else(|| "policy".to_owned()),
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
    use serde_json::json;

    use super::*;

    #[test]
    fn policy_guardrails_extract_approval_evidence_and_audit_trail() {
        let payload = json!({
            "effect": "require_review",
            "actions": ["ck.realm.policy.update"],
            "resource": {
                "required_scope": "ck:scope:realm:01HXY/admin.write",
                "approval_evidence": [{
                    "kind": "human_approval",
                    "evidence_ref": "ck:approval:01HXY",
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
            Some("ck:scope:realm:01HXY/admin.write")
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
        let doc = PolicyDocumentDto {
            policy_id: "ck:policy:01HXY".to_owned(),
            scope: "ck:realm:01HXY".to_owned(),
            policy_type: "ck.realm.policy.update".to_owned(),
            payload: json!({
                "resource": {
                    "name": "Realm policy",
                    "priority": 3,
                    "audit_trail": [{"action": "policy.create"}]
                }
            }),
            active: true,
            updated_at: None,
        };

        let policy = policy_from_document(doc);
        assert_eq!(policy.name, "Realm policy");
        assert_eq!(policy.priority, 3);
        assert_eq!(policy.guardrails.audit_trail.len(), 1);
    }
}
