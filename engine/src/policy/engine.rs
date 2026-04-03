use crate::cyberkube_engine_v1::Decision;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rules: Vec<PolicyRule>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub condition: String,
    pub action: PolicyAction,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyAction {
    Allow,
    Block,
    Alert,
    Quarantine,
}

#[derive(Debug, Clone)]
pub struct PolicyContext {
    pub event_type: String,
    pub source: String,
    pub payload: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone)]
pub struct PolicyEngine {
    policies: HashMap<String, Policy>,
    rego_engine: crate::policy::rego::RegoEngine,
    wasm_runtime: crate::policy::wasm::WasmRuntime,
}

impl PolicyEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            policies: HashMap::new(),
            rego_engine: crate::policy::rego::RegoEngine::new()?,
            wasm_runtime: crate::policy::wasm::WasmRuntime::new()?,
        })
    }

    pub fn add_policy(&mut self, policy: Policy) -> Result<()> {
        for rule in &policy.rules {
            if is_wasm_module_path(&rule.condition) {
                continue;
            }
            self.rego_engine.compile_policy(&rule.condition)?;
        }
        self.policies.insert(policy.id.clone(), policy);
        Ok(())
    }

    pub fn evaluate(&self, context: &PolicyContext) -> Result<Decision> {
        let mut sorted_policies: Vec<&Policy> = self
            .policies
            .values()
            .filter(|policy| policy.enabled)
            .collect();
        sorted_policies.sort_by_key(|policy| {
            policy
                .rules
                .iter()
                .map(|rule| rule.priority)
                .max()
                .unwrap_or(0)
        });
        sorted_policies.reverse();

        for policy in sorted_policies {
            for rule in &policy.rules {
                if self.evaluate_rule(rule, context)? {
                    return Ok(decision_from_action(&rule.action, policy, rule));
                }
            }
        }

        Ok(Decision {
            action: "allow".to_string(),
            reason: "no_matching_policy".to_string(),
        })
    }

    fn evaluate_rule(&self, rule: &PolicyRule, context: &PolicyContext) -> Result<bool> {
        if is_wasm_module_path(&rule.condition) {
            self.evaluate_wasm(&rule.condition, context)
        } else {
            self.evaluate_rego(&rule.condition, context)
        }
    }

    fn evaluate_wasm(&self, module_path: &str, context: &PolicyContext) -> Result<bool> {
        let input = serde_json::to_vec(&context.to_input())?;
        Ok(self.wasm_runtime.evaluate(module_path, &input)? != 0)
    }

    fn evaluate_rego(&self, query: &str, context: &PolicyContext) -> Result<bool> {
        self.rego_engine.evaluate(query, &context.to_input())
    }
}

impl PolicyContext {
    pub fn to_input(&self) -> serde_json::Value {
        json!({
            "event_type": self.event_type,
            "source": self.source,
            "payload": self.payload,
            "metadata": self.metadata,
        })
    }
}

fn decision_from_action(action: &PolicyAction, policy: &Policy, rule: &PolicyRule) -> Decision {
    let action = match action {
        PolicyAction::Allow => "allow",
        PolicyAction::Block => "block",
        PolicyAction::Alert => "alert",
        PolicyAction::Quarantine => "quarantine",
    };

    Decision {
        action: action.to_string(),
        reason: format!("policy:{} rule:{}", policy.id, rule.id),
    }
}

fn is_wasm_module_path(condition: &str) -> bool {
    let trimmed = condition.trim();
    trimmed.ends_with(".wasm") || trimmed.ends_with(".wat")
}

#[cfg(test)]
mod tests {
    use super::{Policy, PolicyAction, PolicyContext, PolicyEngine, PolicyRule};
    use std::collections::HashMap;

    #[test]
    fn evaluates_matching_rego_style_rule() {
        let mut engine = PolicyEngine::new().unwrap();
        engine
            .add_policy(Policy {
                id: "policy-1".to_string(),
                name: "Block escapes".to_string(),
                description: "Blocks escape attempts".to_string(),
                enabled: true,
                rules: vec![PolicyRule {
                    id: "rule-1".to_string(),
                    condition: "event_type == \"container_escape_attempt\"".to_string(),
                    action: PolicyAction::Block,
                    priority: 100,
                }],
            })
            .unwrap();

        let decision = engine
            .evaluate(&PolicyContext {
                event_type: "container_escape_attempt".to_string(),
                source: "runtime".to_string(),
                payload: HashMap::new(),
                metadata: HashMap::new(),
            })
            .unwrap();

        assert_eq!(decision.action, "block");
    }
}
