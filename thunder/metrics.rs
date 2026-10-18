use lazy_static::lazy_static;
use prometheus::{
    register_counter, register_counter_vec, register_gauge, register_gauge_vec, register_histogram,
    register_histogram_vec, Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramOpts,
    HistogramVec, Opts,
};

lazy_static! {
    pub static ref KAFKA_MESSAGES_FAILED_PARSE: Counter = register_counter!(
        "kafka_messages_failed_parse_total",
        "Total number of Kafka messages that failed to parse"
    )
    .unwrap();
    pub static ref KAFKA_POLL_ERRORS: Counter = register_counter!(
        "kafka_poll_errors_total",
        "Total number of Kafka polling errors"
    )
    .unwrap();
    pub static ref BATCH_PROCESSING_TIME: Histogram = register_histogram!(
        "batch_processing_seconds",
        "Time taken to process a batch in seconds",
        vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]
    )
    .unwrap();
    pub static ref STRATO_REQUESTS: CounterVec = register_counter_vec!(
        Opts::new(
            "strato_requests_total",
            "Total number of Strato requests by endpoint and status"
        ),
        &["endpoint", "status"]
    )
    .unwrap();
    pub static ref STRATO_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        HistogramOpts::new(
            "strato_request_duration_seconds",
            "Strato request duration in seconds"
        )
        .buckets(prometheus::exponential_buckets(0.001, 1.3, 40).unwrap()),
        &["endpoint"]
    )
    .unwrap();
    pub static ref KAFKA_PARTITION_LAG: GaugeVec = register_gauge_vec!(
        Opts::new(
            "kafka_partition_lag",
            "Number of messages behind the high watermark for each partition"
        ),
        &["topic", "partition"]
    )
    .unwrap();
    pub static ref POST_STORE_USER_COUNT: Gauge = register_gauge!(
        "post_store_user_count",
        "Number of unique users in PostStore"
    )
    .unwrap();
    pub static ref POST_STORE_TOTAL_POSTS: Gauge = register_gauge!(
        "post_store_total_posts",
        "Total number of posts in PostStore"
    )
    .unwrap();
    pub static ref POST_STORE_MAX_POSTS_PER_USER: Gauge = register_gauge!(
        "post_store_max_posts_per_user",
        "Maximum number of posts from a single user"
    )
    .unwrap();
    pub static ref POST_STORE_DELETED_POSTS: Gauge = register_gauge!(
        "post_store_deleted_posts",
        "Number of deleted posts in PostStore"
    )
    .unwrap();
    pub static ref POST_STORE_ENTITY_COUNT: GaugeVec = register_gauge_vec!(
        Opts::new(
            "post_store_entity_count",
            "Number of entities in each PostStore data structure"
        ),
        &["type"]
    )
    .unwrap();
    pub static ref POST_STORE_USER_POSTS_DISTRIBUTION: Histogram = register_histogram!(
        "post_store_user_posts_distribution",
        "Distribution of number of posts per user",
        prometheus::exponential_buckets(1.0, 1.3, 55).unwrap()
    )
    .unwrap();
    pub static ref POST_STORE_REPLIES_FILTERED: HistogramVec = register_histogram_vec!(
        HistogramOpts::new(
            "post_store_replies_filtered",
            "Number of replies filtered out during retrieval"
        )
        .buckets(prometheus::exponential_buckets(1.0, 1.3, 55).unwrap()),
        &["reason"]
    )
    .unwrap();
    pub static ref POST_STORE_POSTS_RETURNED: Histogram = register_histogram!(
        "post_store_posts_returned",
        "Number of posts returned per retrieval request",
        prometheus::exponential_buckets(1.0, 1.2, 55).unwrap()
    )
    .unwrap();
    pub static ref GET_IN_NETWORK_POSTS_DURATION: Histogram = register_histogram!(
        "get_in_network_posts_duration_seconds",
        "Total duration of get_in_network_posts requests in seconds",
        prometheus::exponential_buckets(0.001, 1.3, 40).unwrap()
    )
    .unwrap();
    pub static ref GET_IN_NETWORK_POSTS_DURATION_WITHOUT_STRATO: Histogram = register_histogram!(
        "get_in_network_posts_duration_without_strato_seconds",
        "Duration of get_in_network_posts requests excluding strato call in seconds",
        prometheus::exponential_buckets(0.001, 1.3, 40).unwrap()
    )
    .unwrap();
    pub static ref GET_IN_NETWORK_POSTS_COUNT: Histogram = register_histogram!(
        "get_in_network_posts_count",
        "Number of posts returned by get_in_network_posts requests",
        prometheus::exponential_buckets(1.0, 1.2, 55).unwrap()
    )
    .unwrap();
    pub static ref GET_IN_NETWORK_POSTS_FOLLOWING_SIZE: Histogram = register_histogram!(
        "get_in_network_posts_following_size",
        "Number of users in the following list for get_in_network_posts requests",
        prometheus::exponential_buckets(1.0, 1.2, 55).unwrap()
    )
    .unwrap();
    pub static ref GET_IN_NETWORK_POSTS_EXCLUDED_SIZE: Histogram = register_histogram!(
        "get_in_network_posts_excluded_size",
        "Number of tweet IDs in the exclusion list for get_in_network_posts requests",
        prometheus::exponential_buckets(1.0, 1.2, 55).unwrap()
    )
    .unwrap();
    pub static ref GET_IN_NETWORK_POSTS_MAX_RESULTS: Histogram = register_histogram!(
        "get_in_network_posts_max_results",
        "Max results requested for get_in_network_posts requests",
        prometheus::exponential_buckets(1.0, 1.2, 55).unwrap()
    )
    .unwrap();
    pub static ref GET_IN_NETWORK_POSTS_FOUND_FRESHNESS_SECONDS: HistogramVec =
        register_histogram_vec!(
            HistogramOpts::new(
                "get_in_network_posts_found_freshness_seconds",
                "Time in seconds since the most recent post in found results"
            )
            .buckets(vec![
                60.0, 300.0, 600.0, 1800.0, 3600.0, 7200.0, 14400.0, 28800.0, 43200.0, 86400.0,
                129600.0, 172800.0
            ]),
            &["stage"]
        )
        .unwrap();
    pub static ref GET_IN_NETWORK_POSTS_FOUND_TIME_RANGE_SECONDS: HistogramVec =
        register_histogram_vec!(
            HistogramOpts::new(
                "get_in_network_posts_found_time_range_seconds",
                "Time span in seconds between oldest and newest post in found results"
            )
            .buckets(vec![
                60.0, 300.0, 600.0, 1800.0, 3600.0, 7200.0, 14400.0, 28800.0, 43200.0, 86400.0,
                129600.0, 172800.0
            ]),
            &["stage"]
        )
        .unwrap();
    pub static ref GET_IN_NETWORK_POSTS_FOUND_REPLY_RATIO: HistogramVec = register_histogram_vec!(
        HistogramOpts::new(
            "get_in_network_posts_found_reply_ratio",
            "Ratio of replies to total posts in found results"
        )
        .buckets(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]),
        &["stage"]
    )
    .unwrap();
    pub static ref GET_IN_NETWORK_POSTS_FOUND_UNIQUE_AUTHORS: HistogramVec =
        register_histogram_vec!(
            HistogramOpts::new(
                "get_in_network_posts_found_unique_authors",
                "Number of unique authors in found posts"
            )
            .buckets(prometheus::exponential_buckets(1.0, 1.2, 55).unwrap()),
            &["stage"]
        )
        .unwrap();
    pub static ref GET_IN_NETWORK_POSTS_FOUND_POSTS_PER_AUTHOR: HistogramVec =
        register_histogram_vec!(
            HistogramOpts::new(
                "get_in_network_posts_found_posts_per_author",
                "Average number of posts per author in found results"
            )
            .buckets(vec![1.0, 2.0, 3.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0]),
            &["stage"]
        )
        .unwrap();
    pub static ref POST_STORE_REQUEST_TIMEOUTS: Counter = register_counter!(
        "post_store_request_timeouts_total",
        "Total number of request timeouts in PostStore get_posts_by_users"
    )
    .unwrap();
    pub static ref POST_STORE_REQUESTS: Counter = register_counter!(
        "post_store_requests_total",
        "Total number of requests to PostStore get_posts_by_users"
    )
    .unwrap();
    pub static ref POST_STORE_POSTS_RETURNED_RATIO: Histogram = register_histogram!(
        "post_store_posts_returned_ratio",
        "Ratio of posts returned to eligible posts per request",
        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    )
    .unwrap();
    pub static ref IN_FLIGHT_REQUESTS: Gauge = register_gauge!(
        "in_flight_requests",
        "Number of requests currently being processed"
    )
    .unwrap();
    pub static ref REJECTED_REQUESTS: Counter = register_counter!(
        "rejected_requests_total",
        "Total number of requests rejected due to QPS rate limit (RESOURCE_EXHAUSTED)"
    )
    .unwrap();
    pub static ref INIT_DURATION_SECONDS: Gauge = register_gauge!(
        "init_duration_seconds",
        "Time in seconds the Kafka init (catchup) took at startup"
    )
    .unwrap();
}

pub struct Timer {
    histogram: Histogram,
    start: std::time::Instant,
}

impl Timer {
    pub fn new(histogram: Histogram) -> Self {
        Self {
            histogram,
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.histogram.observe(duration.as_secs_f64());
    }
}
