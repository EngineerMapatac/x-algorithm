use crate::processor::ProcessorStats;
use prometheus::{
    CounterVec, GaugeVec, HistogramOpts, HistogramVec, IntCounter, IntGauge, Opts, Registry,
};

#[derive(Clone)]
pub struct PipelineMetrics {
    pub registry: Registry,

    pub records_processed_total: IntCounter,
    pub batch_size: HistogramVec,
    pub batch_poll_duration_seconds: HistogramVec,
    pub batch_process_duration_seconds: HistogramVec,
    pub batch_store_duration_seconds: HistogramVec,

    pub processor_success_total: CounterVec,
    pub processor_filtered_total: CounterVec,
    pub processor_invalid_total: CounterVec,
    pub processor_deser_error_total: CounterVec,
    pub processor_success_rate: GaugeVec,

    pub store_buffer_size: GaugeVec,
    pub store_dump_started: IntCounter,
    pub store_dump_completed: IntCounter,
    pub store_dump_last_success_ts: IntGauge,
    pub store_compaction_removed_total: IntCounter,
    pub store_window_records_written: CounterVec,
    pub store_window_last_dump_records: GaugeVec,
}

impl PipelineMetrics {
    pub fn new(registry: Registry) -> anyhow::Result<Self> {
        Self::new_with_pipeline(registry, "main")
    }

    pub fn new_with_pipeline(registry: Registry, pipeline: &str) -> anyhow::Result<Self> {
        let records_metric_name = if pipeline == "topic" {
            "topic_phoenix_records_processed_total"
        } else {
            "phoenix_records_processed_total"
        };

        let records_processed_total =
            IntCounter::new(records_metric_name, "Total records consumed from Kafka")?;
        registry.register(Box::new(records_processed_total.clone()))?;

        let batch_size = HistogramVec::new(
            HistogramOpts::new("phoenix_batch_size", "Batch size distribution"),
            &[],
        )?;
        registry.register(Box::new(batch_size.clone()))?;

        let batch_poll_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "phoenix_batch_poll_duration_seconds",
                "Time to poll a batch from Kafka",
            ),
            &[],
        )?;
        registry.register(Box::new(batch_poll_duration_seconds.clone()))?;

        let batch_process_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "phoenix_batch_process_duration_seconds",
                "Time to deserialize and transform a batch",
            ),
            &[],
        )?;
        registry.register(Box::new(batch_process_duration_seconds.clone()))?;

        let batch_store_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "phoenix_batch_store_duration_seconds",
                "Time to buffer a batch in the store",
            ),
            &[],
        )?;
        registry.register(Box::new(batch_store_duration_seconds.clone()))?;

        let processor_success_total = CounterVec::new(
            Opts::new(
                "phoenix_processor_success_total",
                "Records successfully processed",
            ),
            &["processor"],
        )?;
        registry.register(Box::new(processor_success_total.clone()))?;

        let processor_filtered_total = CounterVec::new(
            Opts::new(
                "phoenix_processor_ignored_filtered",
                "Records ignored due to filtering (e.g. index_name mismatch)",
            ),
            &["processor"],
        )?;
        registry.register(Box::new(processor_filtered_total.clone()))?;

        let processor_invalid_total = CounterVec::new(
            Opts::new(
                "phoenix_processor_failed_invalid_data",
                "Records dropped due to invalid/missing data",
            ),
            &["processor"],
        )?;
        registry.register(Box::new(processor_invalid_total.clone()))?;

        let processor_deser_error_total = CounterVec::new(
            Opts::new(
                "phoenix_processor_failed_exception",
                "Records dropped due to deserialization or unexpected errors",
            ),
            &["processor"],
        )?;
        registry.register(Box::new(processor_deser_error_total.clone()))?;

        let processor_success_rate = GaugeVec::new(
            Opts::new("phoenix_processor_success_rate", "Processor success rate %"),
            &["processor"],
        )?;
        registry.register(Box::new(processor_success_rate.clone()))?;

        let store_buffer_size = GaugeVec::new(
            Opts::new("phoenix_store_buffer_size", "Records in buffer per window"),
            &["index_name", "window"],
        )?;
        registry.register(Box::new(store_buffer_size.clone()))?;

        let store_dump_started =
            IntCounter::new("phoenix_store_dump_started", "Dump cycles started")?;
        registry.register(Box::new(store_dump_started.clone()))?;

        let store_dump_completed =
            IntCounter::new("phoenix_store_dump_completed", "Dump cycles completed")?;
        registry.register(Box::new(store_dump_completed.clone()))?;

        let store_dump_last_success_ts = IntGauge::new(
            "phoenix_store_dump_last_success_timestamp",
            "Last successful dump epoch seconds",
        )?;
        registry.register(Box::new(store_dump_last_success_ts.clone()))?;

        let store_compaction_removed_total = IntCounter::new(
            "phoenix_store_compaction_removed_total",
            "Records evicted by retention",
        )?;
        registry.register(Box::new(store_compaction_removed_total.clone()))?;

        let store_window_records_written = CounterVec::new(
            Opts::new(
                "phoenix_store_window_records_written_total",
                "Records written per window",
            ),
            &["index_name", "window"],
        )?;
        registry.register(Box::new(store_window_records_written.clone()))?;

        let store_window_last_dump_records = GaugeVec::new(
            Opts::new(
                "phoenix_store_window_last_dump_records",
                "Records in last dump per window",
            ),
            &["index_name", "window"],
        )?;
        registry.register(Box::new(store_window_last_dump_records.clone()))?;

        Ok(Self {
            registry,
            records_processed_total,
            batch_size,
            batch_poll_duration_seconds,
            batch_process_duration_seconds,
            batch_store_duration_seconds,
            processor_success_total,
            processor_filtered_total,
            processor_invalid_total,
            processor_deser_error_total,
            processor_success_rate,
            store_buffer_size,
            store_dump_started,
            store_dump_completed,
            store_dump_last_success_ts,
            store_compaction_removed_total,
            store_window_records_written,
            store_window_last_dump_records,
        })
    }

    pub fn update_processor_stats(&self, label: &str, stats: &ProcessorStats) {
        let success = &self.processor_success_total.with_label_values(&[label]);
        let current = success.get();
        let target = stats.total_success as f64;
        if target > current {
            success.inc_by(target - current);
        }

        let filtered = &self.processor_filtered_total.with_label_values(&[label]);
        let current = filtered.get();
        let target = stats.total_filtered as f64;
        if target > current {
            filtered.inc_by(target - current);
        }

        let invalid = &self.processor_invalid_total.with_label_values(&[label]);
        let current = invalid.get();
        let target = stats.total_invalid as f64;
        if target > current {
            invalid.inc_by(target - current);
        }

        let deser = &self.processor_deser_error_total.with_label_values(&[label]);
        let current = deser.get();
        let target = stats.total_deser_error as f64;
        if target > current {
            deser.inc_by(target - current);
        }

        self.processor_success_rate
            .with_label_values(&[label])
            .set(stats.success_rate());
    }
}
