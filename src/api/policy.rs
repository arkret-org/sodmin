use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::api::client::{api_client, build_url, json_body};
use crate::api::contracts::soland_admin::{CreatePolicyRequest, Policy, PolicyListOutcome};
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
        is_enabled: doc.active,
        priority,
        created_at: None,
        updated_at: doc.updated_at,
    }
}
