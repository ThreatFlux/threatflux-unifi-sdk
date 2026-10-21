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
    let value: JsonValue =
        serde_saphyr::from_str(contents).map_err(|e| UnifiError::ConfigError(e.to_string()))?;
    parse_json_value(value)
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
    fn yaml_config_preserves_typed_values_and_export_roundtrip() {
        let config = parse_yaml(
            r"unifi:
  host: https://controller.invalid
  username: fixture-user
  password: '${TF_ROLLUP_UNSET_CONFIG_VALUE}'
  verify_ssl: true
  timeout_secs: 17
networks:
  - name: 'Lab: #1'
    vlan_enabled: true
    vlan: 42
",
        )
        .expect("valid YAML config");
        assert!(config.unifi.verify_ssl);
        assert_eq!(config.unifi.timeout_secs, 17);
        assert_eq!(config.unifi.site, "default");
        assert_eq!(config.unifi.password, "${TF_ROLLUP_UNSET_CONFIG_VALUE}");
        assert_eq!(config.networks[0].name, "Lab: #1");
        assert_eq!(config.networks[0].vlan, Some(42));
        let exported = serde_saphyr::to_string(&config).expect("YAML export");
        let restored = parse_yaml(&exported).expect("exported YAML config");
        assert_eq!(
            serde_json::to_value(config).expect("JSON config"),
            serde_json::to_value(restored).expect("JSON roundtrip")
        );
    }

    #[test]
    fn yaml_config_rejects_duplicate_keys_and_malformed_input() {
        assert!(parse_yaml("unifi: {host: one, host: two, username: u, password: p}").is_err());
        assert!(parse_yaml("unifi: [").is_err());
    }

    #[test]
    fn substitute_env_keeps_unknown() {
        // SAFETY: This test runs in isolation; no other thread reads TF_TEST_ENV.
        unsafe { std::env::remove_var("TF_TEST_ENV") };
        let result = substitute_env("${TF_TEST_ENV}");
        assert_eq!(result, "${TF_TEST_ENV}");
    }
}
