use crate::{
    cyberkube_engine_v1::{Decision, Policy as ProtoPolicy},
    policy::engine::{Policy, PolicyAction, PolicyContext, PolicyEngine, PolicyRule},
};
use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};
use tracing::{info, warn};

const CONTAINER_ESCAPE_KIND: &str = "container_escape_attempt";

#[derive(Clone, Serialize, Deserialize)]
pub struct PolicyDef {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub wasm_module: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
}

#[derive(Clone)]
pub struct PolicyEngineWasm {
    inner: Arc<RuntimePolicyState>,
}

struct RuntimePolicyState {
    engine: RwLock<PolicyEngine>,
    definitions: RwLock<Vec<PolicyDef>>,
    reload_task: RwLock<Option<tokio::task::JoinHandle<()>>>,
}

impl PolicyEngineWasm {
    pub fn new() -> Result<Self> {
        let definitions = load_policy_definitions()?;
        let engine = build_policy_engine(&definitions)?;
        let state = Arc::new(RuntimePolicyState {
            engine: RwLock::new(engine),
            definitions: RwLock::new(definitions),
            reload_task: RwLock::new(None),
        });
        let reload_task = maybe_spawn_reload_task(state.clone())?;
        if let Ok(mut guard) = state.reload_task.write() {
            *guard = reload_task;
        }

        Ok(Self { inner: state })
    }

    pub fn evaluate_event(&self, kind: &str, payload: &str) -> Result<Decision> {
        let normalized_kind = kind.to_ascii_lowercase();
        let context = build_policy_context(&normalized_kind, payload);
        let decision = self
            .inner
            .engine
            .read()
            .map_err(|_| anyhow!("policy engine lock poisoned"))?
            .evaluate(&context)?;

        if decision.reason != "no_matching_policy" {
            return Ok(decision);
        }

        let (action, reason) = if normalized_kind == CONTAINER_ESCAPE_KIND {
            ("block".to_string(), "default_block_escape".to_string())
        } else {
            ("allow".to_string(), "default_allow".to_string())
        };

        info!("Default policy decision: {action} for kind={normalized_kind}");
        Ok(Decision { action, reason })
    }
}

pub fn load_policies() -> Vec<ProtoPolicy> {
    match load_policy_definitions() {
        Ok(definitions) => definitions.iter().map(proto_policy_from_def).collect(),
        Err(err) => {
            warn!("failed to load policy metadata from disk: {err:#}");
            Vec::new()
        }
    }
}

fn proto_policy_from_def(definition: &PolicyDef) -> ProtoPolicy {
    let wasm_module = first_wasm_rule(definition)
        .cloned()
        .unwrap_or_else(|| definition.wasm_module.clone());

    ProtoPolicy {
        id: definition.id.clone(),
        name: definition.name.clone(),
        description: definition.description.clone(),
        wasm_module,
    }
}

fn build_policy_engine(definitions: &[PolicyDef]) -> Result<PolicyEngine> {
    let mut engine = PolicyEngine::new()?;
    for definition in definitions {
        engine.add_policy(definition_to_runtime_policy(definition)?)?;
    }
    Ok(engine)
}

fn definition_to_runtime_policy(definition: &PolicyDef) -> Result<Policy> {
    let rules = if !definition.rules.is_empty() {
        definition.rules.clone()
    } else if !definition.wasm_module.trim().is_empty() {
        vec![PolicyRule {
            id: format!("{}-wasm", definition.id),
            condition: definition.wasm_module.clone(),
            action: PolicyAction::Block,
            priority: 1,
        }]
    } else {
        bail!(
            "policy '{}' must define at least one rule or a wasm_module",
            definition.id
        );
    };

    Ok(Policy {
        id: definition.id.clone(),
        name: definition.name.clone(),
        description: definition.description.clone(),
        enabled: definition.enabled,
        rules,
    })
}

fn load_policy_definitions() -> Result<Vec<PolicyDef>> {
    let Some(base) = resolve_policy_dir() else {
        info!("no policy directory found; returning empty policy list");
        return Ok(Vec::new());
    };

    let mut policies = Vec::new();
    for entry in fs::read_dir(&base).with_context(|| format!("reading {}", base.display()))? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let data = fs::read_to_string(entry.path())
            .with_context(|| format!("reading policy file {}", entry.path().display()))?;
        let mut definition = serde_json::from_str::<PolicyDef>(&data)
            .with_context(|| format!("parsing policy file {}", entry.path().display()))?;
        normalize_policy_paths(&mut definition, &base);
        validate_policy_definition(&definition)?;
        policies.push(definition);
    }

    policies.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(policies)
}

fn validate_policy_definition(definition: &PolicyDef) -> Result<()> {
    if definition.id.trim().is_empty() {
        bail!("policy id cannot be empty");
    }
    if definition.name.trim().is_empty() {
        bail!("policy '{}' must have a name", definition.id);
    }
    if definition.description.trim().is_empty() {
        bail!("policy '{}' must have a description", definition.id);
    }
    if definition.rules.is_empty() && definition.wasm_module.trim().is_empty() {
        bail!(
            "policy '{}' must include at least one rule or a wasm_module path",
            definition.id
        );
    }

    let mut seen_rule_ids = std::collections::BTreeSet::new();
    for rule in &definition.rules {
        if rule.id.trim().is_empty() {
            bail!(
                "policy '{}' contains a rule with an empty id",
                definition.id
            );
        }
        if !seen_rule_ids.insert(rule.id.clone()) {
            bail!(
                "policy '{}' contains duplicate rule id '{}'",
                definition.id,
                rule.id
            );
        }
        if rule.condition.trim().is_empty() {
            bail!(
                "policy '{}' rule '{}' must have a condition",
                definition.id,
                rule.id
            );
        }
    }

    Ok(())
}

fn normalize_policy_paths(definition: &mut PolicyDef, base: &Path) {
    if !definition.wasm_module.trim().is_empty() {
        definition.wasm_module = normalize_condition_path(&definition.wasm_module, base);
    }

    for rule in &mut definition.rules {
        if is_wasm_path(&rule.condition) {
            rule.condition = normalize_condition_path(&rule.condition, base);
        }
    }
}

fn normalize_condition_path(condition: &str, base: &Path) -> String {
    let path = PathBuf::from(condition.trim());
    if path.is_absolute() || path.exists() {
        return path.display().to_string();
    }

    base.join(path).display().to_string()
}

fn maybe_spawn_reload_task(
    state: Arc<RuntimePolicyState>,
) -> Result<Option<tokio::task::JoinHandle<()>>> {
    let enabled = env_bool("POLICY_RELOAD_ENABLED", true);
    if !enabled {
        return Ok(None);
    }

    let Some(policy_dir) = resolve_policy_dir() else {
        return Ok(None);
    };

    if tokio::runtime::Handle::try_current().is_err() {
        return Ok(None);
    }

    let interval = Duration::from_secs(env_u64("POLICY_RELOAD_INTERVAL_SECS", 5));
    let policy_dir_for_task = policy_dir.clone();

    Ok(Some(tokio::spawn(async move {
        let mut last_fingerprint = fingerprint_policy_dir(&policy_dir_for_task).ok();
        loop {
            tokio::time::sleep(interval).await;
            let Ok(current_fingerprint) = fingerprint_policy_dir(&policy_dir_for_task) else {
                continue;
            };

            if last_fingerprint.as_ref() == Some(&current_fingerprint) {
                continue;
            }

            match load_policy_definitions() {
                Ok(definitions) => match build_policy_engine(&definitions) {
                    Ok(engine) => {
                        if let Ok(mut guard) = state.engine.write() {
                            *guard = engine;
                        }
                        if let Ok(mut guard) = state.definitions.write() {
                            *guard = definitions;
                        }
                        info!(
                            policy_dir = %policy_dir_for_task.display(),
                            "reloaded policy definitions from disk"
                        );
                        last_fingerprint = Some(current_fingerprint);
                    }
                    Err(err) => {
                        warn!("policy reload skipped because engine rebuild failed: {err:#}")
                    }
                },
                Err(err) => {
                    warn!("policy reload skipped because definitions failed validation: {err:#}")
                }
            }
        }
    })))
}

fn build_policy_context(kind: &str, payload: &str) -> PolicyContext {
    let payload_value = serde_json::from_str::<Value>(payload).unwrap_or(Value::Null);
    let payload_map = extract_string_map(payload_value.get("payload").unwrap_or(&payload_value));

    let mut metadata = extract_string_map(payload_value.get("labels").unwrap_or(&Value::Null));

    if let Some(namespace) = payload_value.get("namespace").and_then(value_as_string) {
        metadata.insert("namespace".to_string(), namespace);
    }
    if let Some(pod) = payload_value.get("pod").and_then(value_as_string) {
        metadata.insert("pod".to_string(), pod);
    }
    if let Some(source) = payload_value.get("source").and_then(value_as_string) {
        metadata.insert("source".to_string(), source);
    }

    PolicyContext {
        event_type: kind.to_string(),
        source: payload_value
            .get("source")
            .and_then(value_as_string)
            .unwrap_or_else(|| "runtime".to_string()),
        payload: payload_map,
        metadata,
    }
}

fn extract_string_map(value: &Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Value::Object(object) = value {
        for (key, value) in object {
            if let Some(string_value) = value_as_string(value) {
                map.insert(key.clone(), string_value);
            }
        }
    }
    map
}

fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::Array(_) | Value::Object(_) => Some(value.to_string()),
    }
}

fn resolve_policy_dir() -> Option<PathBuf> {
    if let Ok(path) = env::var("POLICY_DIR") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    let candidates = [
        PathBuf::from("policies"),
        PathBuf::from("engine").join("policies"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("policies"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .map(|path| path.join("policies"))
            .unwrap_or_else(|| PathBuf::from("policies")),
    ];

    candidates.into_iter().find(|path| path.exists())
}

fn first_wasm_rule(definition: &PolicyDef) -> Option<&String> {
    definition.rules.iter().find_map(|rule| {
        let condition = rule.condition.trim();
        if is_wasm_path(condition) {
            Some(&rule.condition)
        } else {
            None
        }
    })
}

fn is_wasm_path(condition: &str) -> bool {
    let trimmed = condition.trim();
    trimmed.ends_with(".wasm") || trimmed.ends_with(".wat")
}

fn fingerprint_policy_dir(path: &Path) -> Result<String> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let modified = metadata.modified().ok();
        entries.push(format!(
            "{}:{}:{:?}",
            entry.file_name().to_string_lossy(),
            metadata.len(),
            modified
        ));
    }
    entries.sort();
    Ok(entries.join("|"))
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|value| match value.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

const fn default_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{load_policy_definitions, PolicyEngineWasm};
    use std::{
        fs,
        sync::{Mutex, OnceLock},
    };
    use tempfile::tempdir;

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn loads_relative_wasm_policy_paths_from_policy_dir() {
        let _guard = env_lock().lock().expect("env lock poisoned");
        let temp = tempdir().expect("create tempdir");
        let policy_dir = temp.path();
        let wat_path = policy_dir.join("demo_policy.wat");
        let json_path = policy_dir.join("demo_policy.json");

        fs::write(
            &wat_path,
            r#"(module
  (memory (export "memory") 1)
  (global $heap (mut i32) (i32.const 1024))
  (func (export "alloc") (param $len i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $heap))
    (global.set $heap (i32.add (global.get $heap) (local.get $len)))
    (local.get $ptr))
  (func (export "evaluate") (param $ptr i32) (param $len i32) (result i32)
    (if (result i32)
      (i32.gt_s (local.get $len) (i32.const 0))
      (then (i32.const 1))
      (else (i32.const 0)))))"#,
        )
        .expect("write wat file");
        fs::write(
            &json_path,
            r#"{
  "id": "wasm-demo",
  "name": "WASM Demo",
  "description": "Demonstrates loading a relative WAT policy",
  "enabled": true,
  "rules": [
    {
      "id": "wasm-demo-rule",
      "condition": "demo_policy.wat",
      "action": "Alert",
      "priority": 90
    }
  ]
}"#,
        )
        .expect("write policy file");

        std::env::set_var("POLICY_DIR", policy_dir);
        std::env::set_var("POLICY_RELOAD_ENABLED", "false");

        let definitions = load_policy_definitions().expect("load policy definitions");
        assert_eq!(definitions.len(), 1);
        assert!(definitions[0].rules[0]
            .condition
            .ends_with("demo_policy.wat"));

        let engine = PolicyEngineWasm::new().expect("build runtime policy engine");
        let decision = engine
            .evaluate_event(
                "connect",
                r#"{"source":"runtime","payload":{"message":"demo"}}"#,
            )
            .expect("evaluate event");
        assert_eq!(decision.action, "alert");

        std::env::remove_var("POLICY_RELOAD_ENABLED");
        std::env::remove_var("POLICY_DIR");
    }
}
