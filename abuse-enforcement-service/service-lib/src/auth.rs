use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::{info, warn, Instrument};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct KeyId(pub String);

#[derive(Debug, Default, Clone)]
pub struct ApiKeys(Arc<HashMap<String, String>>);

impl ApiKeys {
    pub fn from_config(cfg: &Config) -> Result<Self> {
        let json = match cfg.api_keys_json.as_deref().map(str::trim) {
            Some(s) if !s.is_empty() => s,
            _ => return Ok(Self::default()),
        };
        let raw: HashMap<String, String> =
            serde_json::from_str(json).context("failed to parse API_KEYS_JSON as a JSON object")?;
        let map: HashMap<String, String> = raw.into_iter().filter(|(_, v)| !v.is_empty()).collect();
        Ok(Self(Arc::new(map)))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn configured_key_ids(&self) -> Vec<String> {
        let mut names: Vec<String> = self.0.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn lookup(&self, presented: &str) -> Option<String> {
        let pb = presented.as_bytes();
        let mut matched: Option<String> = None;
        for (id, secret) in self.0.iter() {
            if constant_time_eq(secret.as_bytes(), pb) {
                matched = Some(id.clone());
            }
        }
        matched
    }
}

pub async fn require_api_key(
    State(keys): State<ApiKeys>,
    mut req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    if keys.is_empty() {
        warn!(%method, %uri, "rejecting request: no API keys configured (fail-closed)");
        return unauthorized();
    }

    let presented = match extract_bearer(req.headers()) {
        Some(t) => t,
        None => {
            warn!(%method, %uri, "rejecting request: missing or malformed Authorization header");
            return unauthorized();
        }
    };

    let key_id = match keys.lookup(&presented) {
        Some(id) => id,
        None => {
            warn!(%method, %uri, "rejecting request: token did not match any configured key");
            return unauthorized();
        }
    };

    req.extensions_mut().insert(KeyId(key_id.clone()));

    let span = tracing::info_span!("authorized_request", key_id = %key_id);
    async move {
        info!(%method, %uri, "request authorized");
        next.run(req).await
    }
    .instrument(span)
    .await
}

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    let header_val = headers.get(header::AUTHORIZATION)?;
    let header_str = header_val.to_str().ok()?;
    let rest = header_str.strip_prefix("Bearer ")?;
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(
            header::WWW_AUTHENTICATE,
            r#"Bearer realm="xai-abuse-enforcement""#,
        )],
        "unauthorized\n",
    )
        .into_response()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(rows: &[(&str, &str)]) -> ApiKeys {
        let map: HashMap<String, String> = rows
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        ApiKeys(Arc::new(map))
    }

    #[test]
    fn empty_keys_is_empty() {
        assert!(ApiKeys::default().is_empty());
    }

    #[test]
    fn configured_key_ids_lists_names_only_sorted() {
        let keys = mk(&[("slackbot", "tok-s"), ("dev", "tok-d"), ("pv2", "tok-p")]);
        assert_eq!(keys.configured_key_ids(), vec!["dev", "pv2", "slackbot"]);
    }

    #[test]
    fn lookup_matches_owning_key_id() {
        let keys = mk(&[("dev", "tok-d"), ("slackbot", "tok-s"), ("pv2", "tok-p")]);
        assert_eq!(keys.lookup("tok-d").as_deref(), Some("dev"));
        assert_eq!(keys.lookup("tok-s").as_deref(), Some("slackbot"));
        assert_eq!(keys.lookup("tok-p").as_deref(), Some("pv2"));
    }

    #[test]
    fn lookup_returns_none_for_unknown_token() {
        let keys = mk(&[("dev", "tok-d")]);
        assert_eq!(keys.lookup("nope"), None);
        assert_eq!(keys.lookup(""), None);
    }

    #[test]
    fn constant_time_eq_matches_only_when_equal() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }

    #[test]
    fn from_config_parses_json_map() {
        let mut cfg: Config = clap::Parser::parse_from::<_, &str>(["bin"]);
        cfg.api_keys_json = Some(r#"{"dev":"tok-d","slackbot":"tok-s","pv2":"tok-p"}"#.to_string());
        let keys = ApiKeys::from_config(&cfg).expect("valid JSON parses");
        assert_eq!(keys.configured_key_ids(), vec!["dev", "pv2", "slackbot"]);
        assert_eq!(keys.lookup("tok-s").as_deref(), Some("slackbot"));
    }

    #[test]
    fn from_config_drops_empty_token_rows() {
        let mut cfg: Config = clap::Parser::parse_from::<_, &str>(["bin"]);
        cfg.api_keys_json = Some(r#"{"dev":"tok-d","slackbot":""}"#.to_string());
        let keys = ApiKeys::from_config(&cfg).unwrap();
        assert_eq!(keys.configured_key_ids(), vec!["dev"]);
    }

    #[test]
    fn from_config_treats_blank_and_missing_as_empty() {
        for blank in [None, Some("".to_string()), Some("   ".to_string())] {
            let mut cfg: Config = clap::Parser::parse_from::<_, &str>(["bin"]);
            cfg.api_keys_json = blank;
            let keys = ApiKeys::from_config(&cfg).unwrap();
            assert!(keys.is_empty());
        }
    }

    #[test]
    fn from_config_rejects_malformed_json() {
        let mut cfg: Config = clap::Parser::parse_from::<_, &str>(["bin"]);
        cfg.api_keys_json = Some("{not-json".to_string());
        assert!(ApiKeys::from_config(&cfg).is_err());
    }

    #[test]
    fn extract_bearer_parses_canonical_header() {
        let mut h = HeaderMap::new();
        h.insert(header::AUTHORIZATION, "Bearer tok-d".parse().unwrap());
        assert_eq!(extract_bearer(&h).as_deref(), Some("tok-d"));
    }

    #[test]
    fn extract_bearer_rejects_other_schemes_and_missing() {
        let mut h = HeaderMap::new();
        assert_eq!(extract_bearer(&h), None);

        h.insert(header::AUTHORIZATION, "Basic abc".parse().unwrap());
        assert_eq!(extract_bearer(&h), None);

        h.insert(header::AUTHORIZATION, "Bearer   ".parse().unwrap());
        assert_eq!(extract_bearer(&h), None);
    }
}
