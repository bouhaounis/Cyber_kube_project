use anyhow::{anyhow, bail, Result};
use serde_json::Value;

#[derive(Clone)]
pub struct RegoEngine;

impl RegoEngine {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn evaluate(&self, query: &str, input: &Value) -> Result<bool> {
        let expression = parse_expression(query)?;
        evaluate_expression(&expression, input)
    }

    pub fn compile_policy(&self, policy: &str) -> Result<()> {
        let _ = parse_expression(policy)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expression {
    Exists(String),
    Comparison {
        path: String,
        operator: Operator,
        expected: String,
    },
    And(Vec<Expression>),
    Or(Vec<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    Eq,
    NotEq,
    Contains,
    StartsWith,
    EndsWith,
}

fn parse_expression(query: &str) -> Result<Expression> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        bail!("policy query cannot be empty");
    }

    if trimmed.contains("||") {
        let expressions = trimmed
            .split("||")
            .map(parse_expression)
            .collect::<Result<Vec<_>>>()?;
        return Ok(Expression::Or(expressions));
    }

    if trimmed.contains("&&") {
        let expressions = trimmed
            .split("&&")
            .map(parse_expression)
            .collect::<Result<Vec<_>>>()?;
        return Ok(Expression::And(expressions));
    }

    if let Some(path) = trimmed
        .strip_prefix("exists(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let path = path.trim();
        if path.is_empty() {
            bail!("exists() requires a field path");
        }
        return Ok(Expression::Exists(path.to_string()));
    }

    const OPERATORS: [(&str, Operator); 5] = [
        (" starts_with ", Operator::StartsWith),
        (" ends_with ", Operator::EndsWith),
        (" contains ", Operator::Contains),
        (" != ", Operator::NotEq),
        (" == ", Operator::Eq),
    ];

    for (needle, operator) in OPERATORS {
        if let Some((path, expected)) = trimmed.split_once(needle) {
            let path = path.trim();
            let expected = expected.trim().trim_matches('"').trim_matches('\'');
            if path.is_empty() || expected.is_empty() {
                bail!("invalid comparison expression: {trimmed}");
            }
            return Ok(Expression::Comparison {
                path: path.to_string(),
                operator,
                expected: expected.to_string(),
            });
        }
    }

    Err(anyhow!("unsupported policy expression: {trimmed}"))
}

fn evaluate_expression(expression: &Expression, input: &Value) -> Result<bool> {
    match expression {
        Expression::Exists(path) => Ok(resolve_path(input, path).is_some()),
        Expression::Comparison {
            path,
            operator,
            expected,
        } => {
            let Some(value) = resolve_path(input, path) else {
                return Ok(false);
            };
            let actual = value_to_string(value);
            Ok(match operator {
                Operator::Eq => actual == *expected,
                Operator::NotEq => actual != *expected,
                Operator::Contains => actual.contains(expected),
                Operator::StartsWith => actual.starts_with(expected),
                Operator::EndsWith => actual.ends_with(expected),
            })
        }
        Expression::And(expressions) => {
            for expression in expressions {
                if !evaluate_expression(expression, input)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Expression::Or(expressions) => {
            for expression in expressions {
                if evaluate_expression(expression, input)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
    }
}

fn resolve_path<'a>(input: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = input;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::RegoEngine;
    use serde_json::json;

    #[test]
    fn evaluates_simple_equality() {
        let engine = RegoEngine::new().unwrap();
        let input = json!({
            "event_type": "connect",
            "payload": { "message": "runtime alert" },
            "metadata": { "namespace": "security" }
        });

        assert!(engine
            .evaluate("event_type == \"connect\"", &input)
            .unwrap());
        assert!(engine
            .evaluate("payload.message contains \"alert\"", &input)
            .unwrap());
        assert!(engine
            .evaluate(
                "event_type == \"connect\" && metadata.namespace == \"security\"",
                &input
            )
            .unwrap());
    }

    #[test]
    fn supports_exists_and_or() {
        let engine = RegoEngine::new().unwrap();
        let input = json!({
            "event_type": "execve",
            "payload": { "message": "spawned shell" }
        });

        assert!(engine.evaluate("exists(payload.message)", &input).unwrap());
        assert!(engine
            .evaluate(
                "event_type == \"connect\" || event_type == \"execve\"",
                &input
            )
            .unwrap());
    }
}
