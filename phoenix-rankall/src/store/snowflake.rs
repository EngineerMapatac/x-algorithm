const TWITTER_EPOCH_MS: i64 = 1288834974657;

pub fn snowflake_to_timestamp_secs(snowflake_id: i64) -> f64 {
    ((snowflake_id >> 22) + TWITTER_EPOCH_MS) as f64 / 1000.0
}

pub fn timestamp_secs_to_snowflake(timestamp_secs: f64) -> i64 {
    ((timestamp_secs * 1000.0) as i64 - TWITTER_EPOCH_MS) << 22
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let now_secs = 1_700_000_000.0;
        let sf = timestamp_secs_to_snowflake(now_secs);
        let back = snowflake_to_timestamp_secs(sf);
        assert!((back - now_secs).abs() < 0.001);
    }

    #[test]
    fn snowflake_ordering() {
        let earlier = timestamp_secs_to_snowflake(1_700_000_000.0);
        let later = timestamp_secs_to_snowflake(1_700_000_060.0);
        assert!(later > earlier);
    }
}
