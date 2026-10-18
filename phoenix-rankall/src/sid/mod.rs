pub mod backfill;
pub mod client;
pub mod metrics;

pub mod proto {
    tonic::include_proto!("sid_lookup");
}

const TWITTER_EPOCH_MS: i64 = 1288834974657;

pub fn post_age_seconds(post_id: i64, now_ms: i64) -> f64 {
    if post_id <= 0 {
        return 0.0;
    }
    let created_ms = (post_id >> 22) + TWITTER_EPOCH_MS;
    let age_ms = now_ms.saturating_sub(created_ms);
    (age_ms.max(0)) as f64 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_age_sane() {
        let one_hour_ms: i64 = 3_600_000;
        let post_id = one_hour_ms << 22;
        let now_ms = TWITTER_EPOCH_MS + 2 * one_hour_ms;
        let age = post_age_seconds(post_id, now_ms);
        assert!((age - 3600.0).abs() < 0.001, "got {age}");
    }

    #[test]
    fn post_age_zero_for_invalid_post_id() {
        assert_eq!(post_age_seconds(0, 1), 0.0);
        assert_eq!(post_age_seconds(-1, 1), 0.0);
    }

    #[test]
    fn post_age_zero_for_future_post() {
        let now_ms = TWITTER_EPOCH_MS + 1_000_000;
        let post_id = (now_ms - TWITTER_EPOCH_MS) << 22;
        let earlier_now_ms = now_ms - 1000;
        assert_eq!(post_age_seconds(post_id, earlier_now_ms), 0.0);
    }
}
