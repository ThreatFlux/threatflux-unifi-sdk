//! Config parser with environment substitution.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde_json::Value as JsonValue;

use crate::error::{Result, UnifiError};

use super::schema::UnifiDeclarativeConfig;

/// Load configuration from a file path.
pub fn load_config(path: impl AsRef<Path>) -> Result<UnifiDeclarativeConfig> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .map_err(|e| UnifiError::ConfigError(format!("failed to read config: {e}")))?;

    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("yaml") {
        "json" => parse_json(&raw),
        _ => parse_yaml(&raw),
    }
}

/// Parse YAML config.
pub fn parse_yaml(contents: &str) -> Result<UnifiDeclarativeConfig> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(contents).map_err(|e| UnifiError::ConfigError(e.to_string()))?;
    let json_value = serde_json::to_value(value).map_err(UnifiError::JsonError)?;
    parse_json_value(json_value)
}

/// Parse JSON config.
pub fn parse_json(contents: &str) -> Result<UnifiDeclarativeConfig> {
    let value: JsonValue =
        serde_json::from_str(contents).map_err(|e| UnifiError::ConfigError(e.to_string()))?;
    parse_json_value(value)
}

fn parse_json_value(value: JsonValue) -> Result<UnifiDeclarativeConfig> {
    let resolved = resolve_env(value);
    serde_json::from_value(resolved).map_err(|e| UnifiError::ConfigError(e.to_string()))
}

fn resolve_env(value: JsonValue) -> JsonValue {
    match value {
        JsonValue::String(input) => JsonValue::String(substitute_env(&input)),
        JsonValue::Array(items) => JsonValue::Array(items.into_iter().map(resolve_env).collect()),
        JsonValue::Object(map) => {
            JsonValue::Object(map.into_iter().map(|(k, v)| (k, resolve_env(v))).collect())
        }
        other => other,
    }
}

fn substitute_env(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next();
            let mut var = String::new();
            for next in chars.by_ref() {
                if next == '}' {
                    break;
                }
                var.push(next);
            }
            if var.is_empty() {
                output.push_str("${}");
            } else if let Ok(value) = std::env::var(&var) {
                output.push_str(&value);
            } else {
                let _ = write!(output, "${{{var}}}");
            }
        } else {
            output.push(ch);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitute_env_keeps_unknown() {
        // SAFETY: This test runs in isolation; no other thread reads TF_TEST_ENV.
        unsafe { std::env::remove_var("TF_TEST_ENV") };
        let result = substitute_env("${TF_TEST_ENV}");
        assert_eq!(result, "${TF_TEST_ENV}");
    }
}
