use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use reqwest::Client;
use serde_json::{json, Map, Value};
use std::time::Duration;
use tracing::{info, warn};

const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct GrowthBookWriter {
    api_base: String,
    api_key: String,
    client: Client,
    write_lock: Arc<tokio::sync::Mutex<()>>,
}

impl GrowthBookWriter {
    pub fn new(api_host: &str, api_key: String) -> Self {
        let api_base = format!("{}/api/v1", api_host.trim_end_matches('/'));
        let client = Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
            .expect("reqwest client builder should succeed with defaults");
        Self {
            api_base,
            api_key,
            client,
            write_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub async fn get_feature(&self, feature_key: &str) -> Result<Value> {
        let url = format!("{}/features/{}", self.api_base, feature_key);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?
            .error_for_status()
            .with_context(|| format!("GET {url} returned non-2xx"))?;
        let body: Value = resp
            .json()
            .await
            .with_context(|| format!("decoding {url}"))?;
        body.get("feature")
            .cloned()
            .ok_or_else(|| anyhow!("GET {url} response missing 'feature' field"))
    }

    pub async fn get_env_value(&self, feature_key: &str, environment: &str) -> Result<Value> {
        let feature = self.get_feature(feature_key).await?;
        parse_env_value(&feature, environment)
    }

    pub async fn set_env_value(
        &self,
        feature_key: &str,
        environment: &str,
        value: Value,
    ) -> Result<Value> {
        let feature = self.get_feature(feature_key).await?;
        let body = build_env_update_body(&feature, environment, &value)?;

        let url = format!("{}/features/{}", self.api_base, feature_key);
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("POST {url}"))?;

        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            warn!("POST {url} -> {status} body={body_text}");
            bail!("GrowthBook POST {url} failed: {status} {body_text}");
        }
        info!("GrowthBook set_env_value ok (feature={feature_key} env={environment})");
        Ok(value)
    }

    pub async fn set_env_field(
        &self,
        feature_key: &str,
        environment: &str,
        field: &str,
        value: Value,
    ) -> Result<Value> {
        let _guard = self.write_lock.lock().await;
        let current = self.get_env_value(feature_key, environment).await?;
        let Value::Object(mut current_obj) = current else {
            bail!(
                "environment '{}' value is not a JSON object — can't patch field '{}'",
                environment,
                field
            );
        };
        current_obj.insert(field.to_string(), value);
        let new_value = Value::Object(current_obj);
        self.set_env_value(feature_key, environment, new_value.clone())
            .await?;
        Ok(new_value)
    }
}

fn parse_env_value(feature: &Value, environment: &str) -> Result<Value> {
    let env = feature
        .get("environments")
        .and_then(|e| e.get(environment))
        .ok_or_else(|| anyhow!("environment '{}' not found in feature", environment))?;
    let rules = env
        .get("rules")
        .and_then(|r| r.as_array())
        .ok_or_else(|| anyhow!("no rules in environment '{}'", environment))?;
    let first = rules
        .first()
        .ok_or_else(|| anyhow!("rules array is empty in environment '{}'", environment))?;
    let raw = first.get("value").cloned().unwrap_or(Value::Null);

    if let Value::String(s) = &raw {
        serde_json::from_str(s).map_err(|e| {
            anyhow!(
                "rule value for environment '{}' is not valid JSON: {}",
                environment,
                e
            )
        })
    } else {
        Ok(raw)
    }
}

fn build_env_update_body(feature: &Value, environment: &str, new_value: &Value) -> Result<Value> {
    let env = feature
        .get("environments")
        .and_then(|e| e.get(environment))
        .ok_or_else(|| anyhow!("environment '{}' not found in feature", environment))?;
    let mut rules = env
        .get("rules")
        .and_then(|r| r.as_array())
        .cloned()
        .ok_or_else(|| anyhow!("no rules in environment '{}'", environment))?;
    if rules.is_empty() {
        bail!("rules array is empty in environment '{}'", environment);
    }

    let encoded = serde_json::to_string(new_value)?;
    if let Some(obj) = rules[0].as_object_mut() {
        obj.insert("value".to_string(), Value::String(encoded));
    } else {
        bail!(
            "first rule in environment '{}' is not a JSON object",
            environment
        );
    }

    let enabled = env.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

    let mut env_obj = Map::new();
    env_obj.insert("enabled".to_string(), json!(enabled));
    env_obj.insert("rules".to_string(), Value::Array(rules));

    Ok(json!({
        "environments": {
            environment: Value::Object(env_obj),
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_feature() -> Value {
        json!({
            "id": "xai_abuse_enforcement_service_config",
            "environments": {
                "production": {
                    "enabled": true,
                    "rules": [
                        {
                            "type": "force",
                            "value": "{\"dry_run\":false,\"max_enforcements_per_day\":600000}",
                            "id": "abc"
                        }
                    ]
                },
                "staging": {
                    "enabled": false,
                    "rules": []
                }
            }
        })
    }

    #[test]
    fn parse_env_value_decodes_json_string() {
        let v = parse_env_value(&sample_feature(), "production").unwrap();
        assert_eq!(v["dry_run"], json!(false));
        assert_eq!(v["max_enforcements_per_day"], json!(600000));
    }

    #[test]
    fn parse_env_value_errors_on_missing_environment() {
        let err = parse_env_value(&sample_feature(), "nonexistent").unwrap_err();
        assert!(err.to_string().contains("nonexistent"));
    }

    #[test]
    fn parse_env_value_errors_on_empty_rules() {
        let err = parse_env_value(&sample_feature(), "staging").unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn build_env_update_body_encodes_value_as_json_string() {
        let body = build_env_update_body(
            &sample_feature(),
            "production",
            &json!({"dry_run": true, "max_enforcements_per_day": 100}),
        )
        .unwrap();
        let stored_value = &body["environments"]["production"]["rules"][0]["value"];
        assert!(matches!(stored_value, Value::String(_)));
        let s = stored_value.as_str().unwrap();
        let parsed: Value = serde_json::from_str(s).unwrap();
        assert_eq!(parsed["dry_run"], json!(true));
        assert_eq!(parsed["max_enforcements_per_day"], json!(100));
    }

    #[test]
    fn build_env_update_body_preserves_enabled_and_rule_metadata() {
        let body = build_env_update_body(&sample_feature(), "production", &json!({"any": "value"}))
            .unwrap();
        assert_eq!(body["environments"]["production"]["enabled"], json!(true));
        assert_eq!(
            body["environments"]["production"]["rules"][0]["id"],
            json!("abc")
        );
        assert_eq!(
            body["environments"]["production"]["rules"][0]["type"],
            json!("force")
        );
    }

    #[test]
    fn build_env_update_body_errors_on_missing_environment() {
        let err = build_env_update_body(&sample_feature(), "missing", &json!({})).unwrap_err();
        assert!(err.to_string().contains("missing"));
    }
}
