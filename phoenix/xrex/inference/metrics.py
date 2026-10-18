# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import time

from prometheus_client import CollectorRegistry, Counter, Gauge, Histogram, start_http_server

_JAX_CACHE_LISTENERS_WIRED = False


class MetricsPublisher:
    def __init__(self, port=8000):
        self.custom_registry = CollectorRegistry()
        self.model_requests_total = Counter(
            "model_requests_total", "Total number of model requests", registry=self.custom_registry
        )
        self.model_requests_failed = Counter(
            "model_requests_failed",
            "Total number of model requests that failed",
            registry=self.custom_registry,
        )
        self.model_requests_success = Counter(
            "model_requests_success",
            "Total number of model requests that succeeded",
            registry=self.custom_registry,
        )
        self.model_latency = Histogram(
            "model_latency_seconds",
            "Model request latency",
            labelnames=["step"],
            registry=self.custom_registry,
        )

        self.model_requests_nan_detected = Counter(
            "model_requests_nan_detected",
            "Total number of model requests that produced NaN values",
            registry=self.custom_registry,
        )

        self.model_checkpoint_load_time = Histogram(
            "model_checkpoint_latency_seconds",
            "Model checkpoint load time",
            registry=self.custom_registry,
            buckets=[i for i in range(0, 210, 10)] + [i for i in range(250, 1050, 50)],
        )

        self.model_checkpoint_load_success_timestamp = Gauge(
            "model_checkpoint_load_success_timestamp",
            "Unix Timestamp of successful checkpoint load event",
            registry=self.custom_registry,
        )

        self.model_checkpoint_timestamp = Gauge(
            "model_checkpoint_timestamp",
            "Unix Timestamp of the latest model checkpoint",
            registry=self.custom_registry,
        )
        self.model_checkpoint_load_failure_timestamp = Gauge(
            "model_checkpoint_load_failure_timestamp",
            "Unix Timestamp of failed checkpoint load event",
            registry=self.custom_registry,
        )

        self.model_checkpoint_elapsed_samples = Gauge(
            "model_checkpoint_elapsed_samples",
            "Elapsed samples of the currently loaded model checkpoint",
            registry=self.custom_registry,
        )

        self.pod_start_timestamp = Gauge(
            "pod_start_timestamp",
            "Unix timestamp when the inference process started "
            "(pod uptime = time() - pod_start_timestamp)",
            registry=self.custom_registry,
        )
        self.grpc_server_ready_timestamp = Gauge(
            "grpc_server_ready_timestamp",
            "Unix timestamp when the gRPC server first began serving "
            "(gRPC uptime = time() - grpc_server_ready_timestamp)",
            registry=self.custom_registry,
        )

        self.pod_status = Gauge(
            "pod_status",
            "Current pod lifecycle status (use label 'status' to filter)",
            labelnames=["status"],
            registry=self.custom_registry,
        )
        for s in (
            "initializing",
            "serving",
            "draining",
            "reloading_checkpoint",
            "warming_up",
            "error",
        ):
            self.pod_status.labels(status=s).set(0)

        self.checkpoint_reload_step_seconds = Histogram(
            "checkpoint_reload_step_seconds",
            "Time spent on each sub-step of a checkpoint reload",
            labelnames=["step"],
            registry=self.custom_registry,
            buckets=[0.1, 0.5, 1, 2, 5, 10, 20, 30, 60, 90, 120, 180, 300, 600],
        )
        self.checkpoint_reload_method = Counter(
            "checkpoint_reload_method_total",
            "Count of checkpoint reloads by method",
            labelnames=["method"],
            registry=self.custom_registry,
        )

        self.checkpoint_reload_success_total = Counter(
            "checkpoint_reload_success_total",
            "Total number of successful checkpoint reloads",
            labelnames=["method"],
            registry=self.custom_registry,
        )
        self.checkpoint_reload_failure_total = Counter(
            "checkpoint_reload_failure_total",
            "Total number of failed checkpoint reloads",
            labelnames=["method"],
            registry=self.custom_registry,
        )

        self.checkpoint_reload_skipped_total = Counter(
            "checkpoint_reload_skipped_total",
            "Reload requests that found no newer checkpoint (invalid / no-op reloads)",
            labelnames=["reason", "drained"],
            registry=self.custom_registry,
        )

        self.copy_port_failure_total = Counter(
            "copy_port_failure_total",
            "Total number of copy_port failures by reason",
            labelnames=["reason"],
            registry=self.custom_registry,
        )

        self.checkpoint_reload_bytes = Gauge(
            "checkpoint_reload_bytes",
            "Bytes transferred during the last checkpoint reload sub-step",
            labelnames=["step"],
            registry=self.custom_registry,
        )

        self.checkpoint_reload_throughput_bytes_per_second = Gauge(
            "checkpoint_reload_throughput_bytes_per_second",
            "Transfer throughput in bytes/sec during the last checkpoint reload sub-step",
            labelnames=["step"],
            registry=self.custom_registry,
        )

        self.inference_startup_seconds = Gauge(
            "inference_startup_seconds",
            "Cold-start duration per phase (init, checkpoint_load, "
            "mm_preload_wait, warmup, total); set once when the server becomes "
            "ready. Excludes container scheduling and image pull.",
            labelnames=["phase"],
            registry=self.custom_registry,
        )

        self.hotswap_emb_table_swap_timestamp = Gauge(
            "hotswap_emb_table_swap_timestamp",
            "Unix timestamp of the last embedding table hot-swap",
            registry=self.custom_registry,
        )
        self.hotswap_dense_weights_swap_timestamp = Gauge(
            "hotswap_dense_weights_swap_timestamp",
            "Unix timestamp of the last dense weights hot-swap",
            registry=self.custom_registry,
        )
        self.hotswap_h2d_copy_seconds = Histogram(
            "hotswap_h2d_copy_seconds",
            "Time spent on the H2D copy during a hot-swap of dense weights",
            registry=self.custom_registry,
            buckets=[0.01, 0.05, 0.1, 0.2, 0.5, 1, 2, 5, 10],
        )
        self.hotswap_background_load_seconds = Histogram(
            "hotswap_background_load_seconds",
            "Total time spent on background checkpoint loading (dense + emb_table)",
            registry=self.custom_registry,
            buckets=[1, 5, 10, 20, 30, 60, 90, 120, 180, 300, 600],
        )
        self.hotswap_total_swap_seconds = Histogram(
            "hotswap_total_swap_seconds",
            "End-to-end swap time (emb_table pointer swap + H2D dense copy)",
            registry=self.custom_registry,
            buckets=[0.01, 0.05, 0.1, 0.2, 0.5, 1, 2, 5, 10],
        )
        self.hotswap_active_slot = Gauge(
            "hotswap_active_slot",
            "Currently active double-buffer slot index (0 or 1)",
            registry=self.custom_registry,
        )
        self.hotswap_success_total = Counter(
            "hotswap_success_total",
            "Total number of successful hot-swaps",
            registry=self.custom_registry,
        )
        self.hotswap_failure_total = Counter(
            "hotswap_failure_total",
            "Total number of failed hot-swap attempts",
            labelnames=["stage"],
            registry=self.custom_registry,
        )
        for s in (
            "background_load",
            "subprocess_crash",
            "live_stage",
            "live_swap",
            "leader_error",
            "swap",
        ):
            self.hotswap_failure_total.labels(stage=s)

        self.retrieval_posts_before_history_filter = Counter(
            "retrieval_posts_before_history_filter",
            "Total number of retrieval posts before history filter",
            registry=self.custom_registry,
        )
        self.retrieval_posts_history_filtered = Counter(
            "retrieval_posts_history_filtered",
            "Total number of retrieval posts filtered out due to user history",
            registry=self.custom_registry,
        )

        self.mm_retrieval_requests_total = Counter(
            "mm_retrieval_requests_total",
            "Total number of MM retrieval user requests processed",
            registry=self.custom_registry,
        )
        self.mm_retrieval_requests_no_seed_total = Counter(
            "mm_retrieval_requests_no_seed_total",
            "MM retrieval user requests with no valid query seed (retrieved nothing)",
            registry=self.custom_registry,
        )
        self.mm_retrieval_seeds_per_request = Histogram(
            "mm_retrieval_seeds_per_request",
            "Valid query seeds per MM retrieval request, over requests with >= 1 seed",
            registry=self.custom_registry,
            buckets=[1, 2, 3, 4, 5, 6, 8, 10, 15, 20],
        )

        self.jax_compilation_cache_hits_total = Counter(
            "jax_compilation_cache_hits_total",
            "JAX persistent compilation cache hits (executable loaded from cache)",
            registry=self.custom_registry,
        )
        self.jax_compilation_cache_misses_total = Counter(
            "jax_compilation_cache_misses_total",
            "JAX persistent compilation cache misses (executable compiled from scratch)",
            registry=self.custom_registry,
        )
        self.jax_compilation_compile_time_saved_seconds_total = Counter(
            "jax_compilation_compile_time_saved_seconds_total",
            "Cumulative wall-clock XLA compile time avoided by cache hits",
            registry=self.custom_registry,
        )
        self.jax_compilation_cache_retrieval_seconds_total = Counter(
            "jax_compilation_cache_retrieval_seconds_total",
            "Cumulative time spent reading executables from the persistent cache",
            registry=self.custom_registry,
        )

        self.model_warmup_seconds = Histogram(
            "model_warmup_seconds",
            "Warmup forward-pass time (JIT compile on cold start, cache load on warm start)",
            labelnames=["phase"],
            registry=self.custom_registry,
            buckets=[1, 5, 10, 20, 30, 60, 90, 120, 180, 300, 600, 1200, 1800, 3600],
        )

        self.inference_original_batch_size: Histogram | None = None
        self.inference_throughput_seqs_per_second: Histogram | None = None

        self.inference_selected_bucket_size: Histogram | None = None
        self.inference_bucket_requests_total: Counter | None = None
        self.inference_bucket_utilization: Histogram | None = None
        self.inference_bucket_padding_rows: Histogram | None = None

        self.port = port

        self._wire_jax_compilation_cache_metrics()

        start_http_server(self.port, registry=self.custom_registry)

    def _wire_jax_compilation_cache_metrics(self) -> None:
        global _JAX_CACHE_LISTENERS_WIRED
        if _JAX_CACHE_LISTENERS_WIRED:
            return
        try:
            from jax._src import monitoring

            def _on_event(event: str, **_kwargs) -> None:
                if event == "/jax/compilation_cache/cache_hits":
                    self.jax_compilation_cache_hits_total.inc()
                elif event == "/jax/compilation_cache/cache_misses":
                    self.jax_compilation_cache_misses_total.inc()

            def _on_duration(event: str, duration_secs: float, **_kwargs) -> None:
                secs = max(0.0, duration_secs)
                if event == "/jax/compilation_cache/compile_time_saved_sec":
                    self.jax_compilation_compile_time_saved_seconds_total.inc(secs)
                elif event == "/jax/compilation_cache/cache_retrieval_time_sec":
                    self.jax_compilation_cache_retrieval_seconds_total.inc(secs)

            monitoring.register_event_listener(_on_event)
            monitoring.register_event_duration_secs_listener(_on_duration)
            _JAX_CACHE_LISTENERS_WIRED = True
        except Exception as e:
            import logging

            logging.getLogger(__name__).warning(
                "Could not wire JAX compilation-cache metrics: %s", e
            )

    def register_inference_batch_metrics(
        self, inference_batch_size: int, buckets: tuple[int, ...] | None = None
    ) -> None:
        batch_step = max(1, inference_batch_size // 16)
        batch_buckets = [batch_step * i for i in range(1, 17)]
        self.inference_original_batch_size = Histogram(
            "inference_original_batch_size",
            "Original (pre-padding) number of requests per inference batch",
            registry=self.custom_registry,
            buckets=batch_buckets,
        )
        self.inference_throughput_seqs_per_second = Histogram(
            "inference_throughput_seqs_per_second",
            "Inference throughput in sequences per second per batch",
            registry=self.custom_registry,
            buckets=[10, 25, 50, 100, 150, 200, 300, 400, 500, 750, 1000, 1500, 2000, 3000, 5000],
        )

        bucket_sizes = tuple(sorted(buckets)) if buckets else (inference_batch_size,)
        self.inference_selected_bucket_size = Histogram(
            "inference_selected_bucket_size",
            "Selected (post-padding) bucket batch size each request was routed to",
            registry=self.custom_registry,
            buckets=list(bucket_sizes),
        )
        self.inference_bucket_requests_total = Counter(
            "inference_bucket_requests_total",
            "Requests routed to each pre-compiled batch-size bucket, by bucket size",
            labelnames=["bucket"],
            registry=self.custom_registry,
        )
        for b in bucket_sizes:
            self.inference_bucket_requests_total.labels(bucket=str(b))
        self.inference_bucket_utilization = Histogram(
            "inference_bucket_utilization",
            "Fraction of the padded bucket actually used "
            "(original request size / selected bucket size); 1.0 == no padding waste",
            registry=self.custom_registry,
            buckets=[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
        )
        self.inference_bucket_padding_rows = Histogram(
            "inference_bucket_padding_rows",
            "Padded (wasted) rows per batch = selected bucket size - original request size",
            registry=self.custom_registry,
            buckets=batch_buckets,
        )

    def observe_bucket_selection(self, orig_batch_size: int, bucket_size: int) -> None:
        if self.inference_bucket_requests_total is None:
            return
        assert self.inference_selected_bucket_size is not None
        assert self.inference_bucket_utilization is not None
        assert self.inference_bucket_padding_rows is not None
        self.inference_bucket_requests_total.labels(bucket=str(bucket_size)).inc()
        self.inference_selected_bucket_size.observe(bucket_size)
        self.inference_bucket_padding_rows.observe(max(0, bucket_size - orig_batch_size))
        if bucket_size > 0:
            self.inference_bucket_utilization.observe(orig_batch_size / bucket_size)

    def set_pod_status(self, status: str) -> None:
        for s in (
            "initializing",
            "serving",
            "draining",
            "reloading_checkpoint",
            "warming_up",
            "error",
        ):
            self.pod_status.labels(status=s).set(1 if s == status else 0)

    def increment_requests(self, stage):
        self.model_requests_total.labels(stage=stage).inc()

    def observe_latency(self, stage):
        def decorator(func):
            def wrapper(*args, **kwargs):
                start_time = time.time()
                result = func(*args, **kwargs)
                self.model_latency.labels(stage=stage).observe(time.time() - start_time)
                return result

            return wrapper

        return decorator


def get_metrics_publisher(port=8000):
    metrics_publisher = MetricsPublisher(port=port)
    return metrics_publisher
