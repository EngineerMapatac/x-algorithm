use std::collections::HashSet;
use std::sync::LazyLock;

const TEST_USER_ID_MASK: i64 = 1 << 11;

const MINIMUM_TEST_USER_ID: i64 = 1 << 53;

static LEGACY_STRESS_TEST_USER_IDS: LazyLock<HashSet<i64>> = LazyLock::new(|| {
    include_str!("../resources/parrot_uids.txt")
        .lines()
        .filter_map(|l| l.trim().parse::<i64>().ok())
        .collect()
});

pub fn is_test_user_id(user_id: i64) -> bool {
    is_gizmoduck_test_user(user_id) || is_legacy_test_user(user_id)
}

fn is_gizmoduck_test_user(user_id: i64) -> bool {
    user_id >= MINIMUM_TEST_USER_ID && (user_id & TEST_USER_ID_MASK) == TEST_USER_ID_MASK
}

fn is_legacy_test_user(user_id: i64) -> bool {
    is_in_legacy_test_users_super_sequence(user_id)
        || LEGACY_STRESS_TEST_USER_IDS.contains(&user_id)
}

fn is_in_legacy_test_users_super_sequence(user_id: i64) -> bool {
    (2_000_000_000..=2_002_999_999).contains(&user_id)
        || (2_003_000_000..=2_100_000_000).contains(&user_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cs_integration_test_user_from_the_incident_is_detected() {
        assert!(is_test_user_id(2_050_001_283));
    }

    #[test]
    fn legacy_super_sequence_boundaries() {
        assert!(is_test_user_id(2_000_000_000));
        assert!(is_test_user_id(2_002_999_999));
        assert!(is_test_user_id(2_003_000_000));
        assert!(is_test_user_id(2_100_000_000));
        assert!(!is_test_user_id(1_999_999_999));
        assert!(!is_test_user_id(2_100_000_001));
    }

    #[test]
    fn snowflake_test_bit_detected_only_above_minimum() {
        let snowflake_test = MINIMUM_TEST_USER_ID | TEST_USER_ID_MASK;
        assert!(is_test_user_id(snowflake_test));
        assert!(!is_test_user_id(MINIMUM_TEST_USER_ID));
        assert!(!is_test_user_id(TEST_USER_ID_MASK));
        assert!(!is_test_user_id((1 << 52) | TEST_USER_ID_MASK));
    }

    #[test]
    fn parrot_ids_load_and_match() {
        assert!(is_test_user_id(1_002_379_116));
        assert_eq!(LEGACY_STRESS_TEST_USER_IDS.len(), 30_001);
        assert!(!is_test_user_id(1_002_379_117));
    }

    #[test]
    fn ordinary_user_ids_are_not_test_users() {
        assert!(!is_test_user_id(285_734_886));
        assert!(!is_test_user_id(12));
        assert!(!is_test_user_id(0));
        assert!(!is_test_user_id(-1));
    }
}
