use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::{env, fs};

const HTTP_METHODS: &[&str] = &["get", "post", "put", "patch", "delete"];

const REQUIRED_COAUTH: &[(&str, &str)] = &[
    // viewer is served by soland on its protocol surface.
    ("GET", "/_cokret/self/account/viewer"),
    // coauth admin resources live under coauth's own `/_coauth/admin/*`
    // namespace (renamed from `/_cokret/local/admin` in coauth bc06024).
    ("GET", "/_coauth/admin/audit-feed"),
    ("GET", "/_coauth/admin/oauth-sessions"),
    ("POST", "/_coauth/admin/oauth-sessions/{id}/finish"),
    ("GET", "/_coauth/admin/personal-sessions"),
    ("POST", "/_coauth/admin/personal-sessions"),
    ("POST", "/_coauth/admin/personal-sessions/{id}/revoke"),
    ("POST", "/_coauth/admin/personal-sessions/{id}/regenerate"),
    ("GET", "/_coauth/admin/upstream-oauth-providers"),
    ("POST", "/_coauth/admin/upstream-oauth-providers"),
    ("DELETE", "/_coauth/admin/upstream-oauth-providers/{id}"),
    (
        "POST",
        "/_coauth/admin/upstream-oauth-providers/{id}/enable",
    ),
    (
        "POST",
        "/_coauth/admin/upstream-oauth-providers/{id}/disable",
    ),
    ("GET", "/_coauth/admin/upstream-oauth-links"),
    ("DELETE", "/_coauth/admin/upstream-oauth-links/{id}"),
    ("GET", "/_coauth/admin/user-registration-tokens"),
    ("POST", "/_coauth/admin/user-registration-tokens"),
    (
        "POST",
        "/_coauth/admin/user-registration-tokens/{id}/revoke",
    ),
    ("GET", "/_coauth/admin/connector-health"),
    ("GET", "/_coauth/admin/notification-channels"),
    ("GET", "/_coauth/admin/notification-templates"),
    ("POST", "/_coauth/admin/notification-templates/publish"),
    ("GET", "/_coauth/admin/accounts"),
    ("GET", "/_coauth/admin/accounts/{id}"),
    ("GET", "/_coauth/admin/accounts/{id}/dids"),
    ("POST", "/_coauth/admin/accounts/{id}/dids"),
    ("DELETE", "/_coauth/admin/accounts/{id}/dids/{did}"),
    ("GET", "/_coauth/admin/accounts/{id}/claims"),
    ("POST", "/_coauth/admin/claims/{id}/revoke"),
    ("GET", "/_coauth/admin/accounts/{id}/session-grants"),
    ("GET", "/_coauth/admin/accounts/{id}/risk-action/current"),
    ("GET", "/_coauth/admin/accounts/{id}/risk-action/history"),
    ("POST", "/_coauth/admin/accounts/{id}/risk-action"),
    (
        "POST",
        "/_coauth/admin/accounts/{id}/risk-action/{proposal_id}/approve",
    ),
    (
        "POST",
        "/_coauth/admin/accounts/{id}/risk-action/{proposal_id}/execute",
    ),
    ("POST", "/_coauth/admin/accounts/{id}/lock"),
    ("POST", "/_coauth/admin/accounts/{id}/disable"),
    ("POST", "/_coauth/admin/accounts/{id}/erase"),
    ("POST", "/_coauth/admin/accounts/{id}/reset-recovery"),
    ("GET", "/_coauth/admin/bridge/describe"),
    // soland product-surface integration describe.
    ("GET", "/_soland/self/integration/describe"),
];

const REQUIRED_SOLAND: &[(&str, &str)] = &[
    ("GET", "/_soland/admin/server/info"),
    ("GET", "/_cokret/describe"),
    ("GET", "/_soland/admin/server/stats"),
    ("GET", "/_soland/admin/server/status"),
    ("GET", "/_soland/admin/server/trust-domain"),
    ("PUT", "/_soland/admin/server/trust-domain"),
    ("GET", "/_soland/admin/server/relaxed-window"),
    ("PUT", "/_soland/admin/server/relaxed-window"),
    ("POST", "/_soland/admin/audit/attestation-evidence"),
    ("GET", "/_soland/admin/realms"),
    ("POST", "/_soland/admin/realms"),
    ("GET", "/_soland/admin/realms/{id}"),
    ("DELETE", "/_soland/admin/realms/{id}"),
    ("GET", "/_soland/admin/realms/{id}/members"),
    ("GET", "/_soland/admin/spaces"),
    ("POST", "/_soland/admin/spaces"),
    ("GET", "/_soland/admin/spaces/{id}"),
    ("PUT", "/_soland/admin/spaces/{id}"),
    ("GET", "/_soland/admin/spaces/{id}/hierarchy"),
    // anchorer / anchor-DAG surface completed the space→realm rename; the
    // runtime client (src/api/anchor.rs, signing_key.rs) and soland both
    // mount the `realms/{realm_id}` form. The contract manifest must match.
    ("GET", "/_soland/admin/realms/{realm_id}/anchorer"),
    (
        "POST",
        "/_soland/admin/realms/{realm_id}/anchorer/reconfigure",
    ),
    ("GET", "/_soland/admin/realms/{realm_id}/bottom"),
    ("GET", "/_soland/admin/realms/{realm_id}/anchor-dag"),
    (
        "POST",
        "/_soland/admin/realms/{realm_id}/anchor-dag/compact",
    ),
    ("GET", "/_soland/admin/moderation/reports"),
    ("POST", "/_soland/admin/moderation/reports/{id}/resolve"),
    ("GET", "/_soland/admin/moderation/appeals"),
    ("POST", "/_soland/admin/moderation/appeals/{id}/review"),
    ("POST", "/_soland/admin/moderation/appeals/{id}/decision"),
    ("POST", "/_soland/admin/moderation/appeals/{id}/close"),
    ("GET", "/_soland/admin/federation/status"),
    ("GET", "/_soland/admin/authz/capabilities"),
    (
        "POST",
        "/_soland/admin/authz/capabilities/{grant_id}/revoke",
    ),
    ("GET", "/_soland/admin/realms/{id}/delivery-binding-policy"),
    ("PUT", "/_soland/admin/realms/{id}/delivery-binding-policy"),
    (
        "GET",
        "/_soland/admin/realms/{id}/delivery-binding/handovers",
    ),
    ("POST", "/_soland/admin/realms/{id}/destroy"),
    ("POST", "/_soland/admin/realms/{id}/destroy/retry"),
    // CKP-0007 Circle admin (P3A.3) — circle is a candidate operation
    // (CKP-0014 §5), so per soland it MUST live on the product surface
    // `/_soland/self/circles/*` and MUST NOT be on `/_cokret` until it
    // enters the formal catalog. Match the runtime client (src/api/circles.rs).
    ("GET", "/_soland/self/circles"),
    ("POST", "/_soland/self/circles"),
    ("GET", "/_soland/self/circles/{circle_id}"),
    ("POST", "/_soland/self/circles/{circle_id}/members"),
    (
        "DELETE",
        "/_soland/self/circles/{circle_id}/members/{actor_id}",
    ),
    ("POST", "/_soland/self/circles/{circle_id}/scope-rotate"),
    ("POST", "/_soland/self/circles/{circle_id}/archive"),
    ("POST", "/_soland/self/circles/{circle_id}/tombstone"),
];

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let openapi_dir = env::var_os("SODMIN_OPENAPI_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("target").join("openapi"));
    println!("cargo:rerun-if-env-changed=SODMIN_OPENAPI_DIR");
    println!("cargo:rerun-if-env-changed=SODMIN_OPENAPI_STRICT");
    println!("cargo:rerun-if-env-changed=SODMIN_I18N_STRICT");
    println!("cargo:rerun-if-env-changed=CI");
    println!("cargo:rerun-if-changed={}", openapi_dir.display());

    let coauth_path = openapi_dir.join("coauth.openapi.json");
    let soland_path = openapi_dir.join("soland.openapi.json");
    let coauth = load_or_fallback(&coauth_path, "coauth", REQUIRED_COAUTH);
    let soland = load_or_fallback(&soland_path, "soland", REQUIRED_SOLAND);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("sodmin_openapi_contracts.rs");
    fs::write(out_file, render(&coauth, &soland)).expect("write openapi manifest");

    generate_i18n_tables(&manifest_dir, &out_dir);
}

/// Reads the externalized flat-JSON translation files (`i18n/en.json`,
/// `i18n/zh-CN.json`) and emits a sorted, compile-time static lookup table
/// (`i18n_tables.rs`) into `OUT_DIR`. The generated tables are plain
/// `&'static [(&'static str, &'static str)]` slices so the runtime does zero
/// heap allocation and keeps no `HashMap`; `i18n.rs` resolves keys with a
/// `binary_search_by`.
fn generate_i18n_tables(manifest_dir: &Path, out_dir: &Path) {
    let i18n_dir = manifest_dir.join("i18n");
    let en_path = i18n_dir.join("en.json");
    let zh_path = i18n_dir.join("zh-CN.json");
    println!("cargo:rerun-if-changed={}", en_path.display());
    println!("cargo:rerun-if-changed={}", zh_path.display());

    let en = load_i18n_table(&en_path);
    let zh = load_i18n_table(&zh_path);

    // Consistency check: key-set drift is a hard error in strict/CI mode.
    let en_keys: BTreeSet<&str> = en.iter().map(|(k, _)| k.as_str()).collect();
    let zh_keys: BTreeSet<&str> = zh.iter().map(|(k, _)| k.as_str()).collect();
    let missing_in_zh: Vec<&str> = en_keys.difference(&zh_keys).copied().collect();
    let missing_in_en: Vec<&str> = zh_keys.difference(&en_keys).copied().collect();
    let strict_i18n = i18n_strict();
    if !missing_in_zh.is_empty() {
        let message = format!(
            "{} key(s) present in en.json but missing in zh-CN.json: {}",
            missing_in_zh.len(),
            missing_in_zh.join(", ")
        );
        if strict_i18n {
            panic!("i18n: {message}");
        }
        println!("cargo:warning=i18n: {message}");
    }
    if !missing_in_en.is_empty() {
        let message = format!(
            "{} key(s) present in zh-CN.json but missing in en.json: {}",
            missing_in_en.len(),
            missing_in_en.join(", ")
        );
        if strict_i18n {
            panic!("i18n: {message}");
        }
        println!("cargo:warning=i18n: {message}");
    }

    let mut out = String::new();
    out.push_str("// @generated by sodmin build.rs; do not edit by hand.\n");
    render_i18n_table(&mut out, "I18N_EN", &en);
    render_i18n_table(&mut out, "I18N_ZH_CN", &zh);
    fs::write(out_dir.join("i18n_tables.rs"), out).expect("write i18n tables");
}

fn i18n_strict() -> bool {
    env::var("SODMIN_I18N_STRICT").ok().as_deref() == Some("1")
        || env::var("CI").ok().as_deref() == Some("true")
        || env::var("CI").ok().as_deref() == Some("1")
}

/// Parses a flat `{ "key": "value", ... }` JSON object into a key-sorted
/// vector, ready for `binary_search_by` at runtime.
fn load_i18n_table(path: &Path) -> Vec<(String, String)> {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read i18n file {}: {}", path.display(), err));
    let value: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("parse i18n JSON {}: {}", path.display(), err));
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("i18n file {} is not a JSON object", path.display()));
    let mut entries: Vec<(String, String)> = object
        .iter()
        .map(|(key, val)| {
            let text = val.as_str().unwrap_or_else(|| {
                panic!(
                    "i18n file {} key `{}` is not a string value",
                    path.display(),
                    key
                )
            });
            (key.clone(), text.to_owned())
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries.dedup_by(|a, b| a.0 == b.0);
    entries
}

fn render_i18n_table(out: &mut String, name: &str, entries: &[(String, String)]) {
    out.push_str("pub static ");
    out.push_str(name);
    out.push_str(": &[(&str, &str)] = &[\n");
    for (key, value) in entries {
        out.push_str("    (");
        out.push_str(&quote(key));
        out.push_str(", ");
        out.push_str(&quote(value));
        out.push_str("),\n");
    }
    out.push_str("];\n");
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
            // Degraded manifest: the REQUIRED list is echoed as the
            // operation set. The `fallback:` source prefix makes the
            // degradation machine-readable (openapi_contract tests skip
            // instead of self-certifying green).
            println!(
                "cargo:warning={} OpenAPI snapshot missing at {} — using fallback manifest \
                 (contract NOT verified against upstream; run scripts/local-openapi-snapshot.sh)",
                name,
                path.display()
            );
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
