use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const HTTP_METHODS: &[&str] = &["get", "post", "put", "patch", "delete"];

const REQUIRED_COAUTH: &[(&str, &str)] = &[
    ("GET", "/api/v1/viewer"),
    ("GET", "/api/admin/v1/audit-feed"),
    ("GET", "/api/admin/v1/oauth2-sessions"),
    ("POST", "/api/admin/v1/oauth2-sessions/{id}/finish"),
    ("GET", "/api/admin/v1/personal-sessions"),
    ("POST", "/api/admin/v1/personal-sessions"),
    ("POST", "/api/admin/v1/personal-sessions/{id}/revoke"),
    ("POST", "/api/admin/v1/personal-sessions/{id}/regenerate"),
    ("GET", "/api/admin/v1/upstream-oauth-providers"),
    ("POST", "/api/admin/v1/upstream-oauth-providers"),
    ("DELETE", "/api/admin/v1/upstream-oauth-providers/{id}"),
    ("POST", "/api/admin/v1/upstream-oauth-providers/{id}/enable"),
    (
        "POST",
        "/api/admin/v1/upstream-oauth-providers/{id}/disable",
    ),
    ("GET", "/api/admin/v1/upstream-oauth-links"),
    ("DELETE", "/api/admin/v1/upstream-oauth-links/{id}"),
    ("GET", "/api/admin/v1/user-registration-tokens"),
    ("POST", "/api/admin/v1/user-registration-tokens"),
    ("POST", "/api/admin/v1/user-registration-tokens/{id}/revoke"),
    ("GET", "/api/admin/v1/connector-health"),
    ("GET", "/api/admin/v1/notification-channels"),
    ("GET", "/api/admin/v1/notification-templates"),
    ("POST", "/api/admin/v1/notification-templates/publish"),
    ("GET", "/api/admin/v1/accounts"),
    ("GET", "/api/admin/v1/accounts/{id}"),
    ("GET", "/api/admin/v1/accounts/{id}/dids"),
    ("POST", "/api/admin/v1/accounts/{id}/dids"),
    ("DELETE", "/api/admin/v1/accounts/{id}/dids/{did}"),
    ("GET", "/api/admin/v1/accounts/{id}/claims"),
    ("POST", "/api/admin/v1/claims/{id}/revoke"),
    ("GET", "/api/admin/v1/accounts/{id}/session-grants"),
    ("GET", "/api/admin/v1/accounts/{id}/risk-action/current"),
    ("GET", "/api/admin/v1/accounts/{id}/risk-action/history"),
    ("POST", "/api/admin/v1/accounts/{id}/risk-action"),
    (
        "POST",
        "/api/admin/v1/accounts/{id}/risk-action/{proposal_id}/approve",
    ),
    (
        "POST",
        "/api/admin/v1/accounts/{id}/risk-action/{proposal_id}/execute",
    ),
    ("POST", "/api/admin/v1/accounts/{id}/lock"),
    ("POST", "/api/admin/v1/accounts/{id}/disable"),
    ("POST", "/api/admin/v1/accounts/{id}/erase"),
    ("POST", "/api/admin/v1/accounts/{id}/reset-recovery"),
    ("GET", "/api/admin/v1/bridge/describe"),
    ("GET", "/contrix/api/v1/integration/describe"),
];

const REQUIRED_SOLAND: &[(&str, &str)] = &[
    ("GET", "/api/admin/v1/server/info"),
    ("GET", "/api/v1/server/describe"),
    ("GET", "/api/admin/v1/server/stats"),
    ("GET", "/api/admin/v1/server/status"),
    ("GET", "/api/admin/v1/server/trust-domain"),
    ("PUT", "/api/admin/v1/server/trust-domain"),
    ("GET", "/api/admin/v1/server/relaxed-window"),
    ("PUT", "/api/admin/v1/server/relaxed-window"),
    ("POST", "/api/admin/v1/audit/attestation-evidence"),
    ("GET", "/api/admin/v1/spaces"),
    ("POST", "/api/admin/v1/spaces"),
    ("GET", "/api/admin/v1/spaces/{id}"),
    ("PUT", "/api/admin/v1/spaces/{id}"),
    ("GET", "/api/admin/v1/spaces/{id}/hierarchy"),
    ("GET", "/api/admin/v1/spaces/{id}/anchorer"),
    ("POST", "/api/admin/v1/spaces/{id}/anchorer/reconfigure"),
    ("GET", "/api/admin/v1/spaces/{id}/bottom"),
    ("GET", "/api/admin/v1/spaces/{id}/anchor-dag"),
    ("POST", "/api/admin/v1/spaces/{id}/anchor-dag/compact"),
    ("GET", "/api/admin/v1/moderation/reports"),
    ("POST", "/api/admin/v1/moderation/reports/{id}/resolve"),
    ("GET", "/api/admin/v1/moderation/appeals"),
    ("POST", "/api/admin/v1/moderation/appeals/{id}/review"),
    ("POST", "/api/admin/v1/moderation/appeals/{id}/decision"),
    ("POST", "/api/admin/v1/moderation/appeals/{id}/close"),
    ("GET", "/api/admin/v1/federation/status"),
    ("GET", "/api/admin/v1/authz/capabilities"),
    ("POST", "/api/admin/v1/authz/capabilities/{grant_id}/revoke"),
    ("GET", "/api/admin/v1/realms/{id}/delivery-binding-policy"),
    ("PUT", "/api/admin/v1/realms/{id}/delivery-binding-policy"),
    (
        "GET",
        "/api/admin/v1/realms/{id}/delivery-binding/handovers",
    ),
    ("POST", "/api/admin/v1/realms/{id}/destroy"),
    ("POST", "/api/admin/v1/realms/{id}/destroy/retry"),
    // CXP-0007 Circle admin (P3A.3) — sodmin consumes the full
    // `/api/v1/circles/*` surface. Missing any route here is treated
    // as a contract break and fails the wasm build at build.rs time.
    ("GET", "/api/v1/circles"),
    ("POST", "/api/v1/circles"),
    ("GET", "/api/v1/circles/{circle_id}"),
    ("POST", "/api/v1/circles/{circle_id}/members"),
    ("DELETE", "/api/v1/circles/{circle_id}/members/{actor_id}"),
    ("POST", "/api/v1/circles/{circle_id}/scope-rotate"),
    ("POST", "/api/v1/circles/{circle_id}/archive"),
    ("POST", "/api/v1/circles/{circle_id}/tombstone"),
];

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let openapi_dir = env::var_os("SODMIN_OPENAPI_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("target").join("openapi"));
    println!("cargo:rerun-if-env-changed=SODMIN_OPENAPI_DIR");
    println!("cargo:rerun-if-env-changed=SODMIN_OPENAPI_STRICT");
    println!("cargo:rerun-if-changed={}", openapi_dir.display());

    let coauth_path = openapi_dir.join("coauth.openapi.json");
    let soland_path = openapi_dir.join("soland.openapi.json");
    let coauth = load_or_fallback(&coauth_path, "coauth", REQUIRED_COAUTH);
    let soland = load_or_fallback(&soland_path, "soland", REQUIRED_SOLAND);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("sodmin_openapi_contracts.rs");
    fs::write(out_file, render(&coauth, &soland)).expect("write openapi manifest");
}

struct OperationSet {
    source: String,
    operations: Vec<(String, String)>,
}

fn load_or_fallback(path: &Path, name: &str, required: &[(&str, &str)]) -> OperationSet {
    match fs::read_to_string(path) {
        Ok(raw) => {
            let operations = parse_openapi_operations(&raw)
                .unwrap_or_else(|err| panic!("parse {} OpenAPI {}: {}", name, path.display(), err));
            validate_required(name, &operations, required);
            OperationSet {
                source: path.display().to_string(),
                operations,
            }
        }
        Err(_) => {
            if env::var("SODMIN_OPENAPI_STRICT").ok().as_deref() == Some("1") {
                panic!(
                    "{} OpenAPI snapshot missing at {}; run scripts/local-openapi-snapshot.sh",
                    name,
                    path.display()
                );
            }
            OperationSet {
                source: format!("fallback:{}", path.display()),
                operations: required
                    .iter()
                    .map(|(method, path)| ((*method).to_owned(), (*path).to_owned()))
                    .collect(),
            }
        }
    }
}

fn parse_openapi_operations(raw: &str) -> Result<Vec<(String, String)>, String> {
    let value: serde_json::Value = serde_json::from_str(raw).map_err(|err| err.to_string())?;
    let paths = value
        .get("paths")
        .and_then(|paths| paths.as_object())
        .ok_or_else(|| "missing OpenAPI paths object".to_string())?;
    let mut ops = Vec::new();
    for (path, item) in paths {
        let Some(item) = item.as_object() else {
            continue;
        };
        for method in HTTP_METHODS {
            if item.contains_key(*method) {
                ops.push((method.to_ascii_uppercase(), path.to_owned()));
            }
        }
    }
    ops.sort();
    ops.dedup();
    Ok(ops)
}

fn validate_required(name: &str, operations: &[(String, String)], required: &[(&str, &str)]) {
    let present: BTreeSet<(String, String)> = operations
        .iter()
        .map(|(method, path)| (method.to_ascii_uppercase(), normalize_path(path)))
        .collect();
    let missing: Vec<String> = required
        .iter()
        .filter(|(method, path)| {
            !present.contains(&(method.to_ascii_uppercase(), normalize_path(path)))
        })
        .map(|(method, path)| format!("{method} {path}"))
        .collect();
    if !missing.is_empty() {
        panic!(
            "{} OpenAPI snapshot is missing sodmin operations: {}",
            name,
            missing.join(", ")
        );
    }
}

fn normalize_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut in_param = false;
    for ch in path.chars() {
        match ch {
            '{' => {
                in_param = true;
                out.push_str("{}");
            }
            '}' => in_param = false,
            _ if !in_param => out.push(ch),
            _ => {}
        }
    }
    out
}

fn render(coauth: &OperationSet, soland: &OperationSet) -> String {
    let mut out = String::new();
    out.push_str("// @generated by sodmin build.rs; do not edit by hand.\n");
    out.push_str("pub const COAUTH_OPENAPI_SOURCE: &str = ");
    out.push_str(&quote(&coauth.source));
    out.push_str(";\n");
    out.push_str("pub const SOLAND_OPENAPI_SOURCE: &str = ");
    out.push_str(&quote(&soland.source));
    out.push_str(";\n");
    render_ops(&mut out, "COAUTH_OPENAPI_OPERATIONS", &coauth.operations);
    render_ops(&mut out, "SOLAND_OPENAPI_OPERATIONS", &soland.operations);
    out
}

fn render_ops(out: &mut String, name: &str, operations: &[(String, String)]) {
    out.push_str("pub const ");
    out.push_str(name);
    out.push_str(": &[(&str, &str)] = &[\n");
    for (method, path) in operations {
        out.push_str("    (");
        out.push_str(&quote(method));
        out.push_str(", ");
        out.push_str(&quote(path));
        out.push_str("),\n");
    }
    out.push_str("];\n");
}

fn quote(value: &str) -> String {
    serde_json::to_string(value).expect("quote string")
}
