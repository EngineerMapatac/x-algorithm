use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct StratoResponse<T> {
    pub v: Option<T>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HighPageRankUser {
    pub is_high_page_rank_user: Option<bool>,
    pub user_cred_score: Option<f64>,
    #[serde(deserialize_with = "deserialize_string_or_i64")]
    pub follower_count: Option<i64>,
}

#[derive(Serialize)]
pub struct DedupEntry<'a> {
    pub status: &'a str,
    pub acted_class: u8,
    pub topic: &'a str,
    pub dry_run: bool,
    pub additional_info_map: &'a std::collections::BTreeMap<String, String>,
    pub score_result: xai_abuse_proto::enforcement::ScoreResult,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DedupEntryOwned {
    pub status: String,
    #[serde(default)]
    pub acted_class: u8,
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub additional_info_map: std::collections::BTreeMap<String, String>,
    pub score_result: xai_abuse_proto::enforcement::ScoreResult,
}

fn deserialize_string_or_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let v = Option::<serde_json::Value>::deserialize(deserializer)?;
    match v {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(n)) => n
            .as_i64()
            .map(Some)
            .ok_or_else(|| D::Error::custom("number out of i64 range")),
        Some(serde_json::Value::String(s)) => s
            .parse::<i64>()
            .map(Some)
            .map_err(|_| D::Error::custom(format!("invalid i64 string: {s}"))),
        Some(other) => Err(D::Error::custom(format!(
            "expected number or string, got {other}"
        ))),
    }
}
