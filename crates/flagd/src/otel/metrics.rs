//! OpenTelemetry metrics for flag evaluation
//!
//! Provides metrics instruments for tracking flag evaluation performance and errors.

use opentelemetry::{
    KeyValue, global,
    metrics::{Counter, Histogram, Meter},
};
use std::sync::OnceLock;
use std::time::Instant;

/// Crate version for telemetry attributes
pub const PROVIDER_VERSION: &str = env!("CARGO_PKG_VERSION");

static METER: OnceLock<Meter> = OnceLock::new();

fn get_meter() -> &'static Meter {
    METER.get_or_init(|| global::meter("open-feature-flagd"))
}

/// Metrics instruments for flag evaluation
pub struct EvaluationMetrics {
    evaluation_total: Counter<u64>,
    evaluation_duration: Histogram<f64>,
    evaluation_error_total: Counter<u64>,
}

impl EvaluationMetrics {
    /// Create a new EvaluationMetrics instance
    #[must_use]
    pub fn new() -> Self {
        let meter = get_meter();

        let evaluation_total = meter
            .u64_counter("feature_flag.evaluation_total")
            .with_description("Total number of flag evaluations")
            .with_unit("1")
            .build();

        let evaluation_duration = meter
            .f64_histogram("feature_flag.evaluation_duration")
            .with_description("Duration of flag evaluations in seconds")
            .with_unit("s")
            .build();

        let evaluation_error_total = meter
            .u64_counter("feature_flag.evaluation_error_total")
            .with_description("Total number of failed flag evaluations")
            .with_unit("1")
            .build();

        Self {
            evaluation_total,
            evaluation_duration,
            evaluation_error_total,
        }
    }

    /// Record a successful flag evaluation
    pub fn record_evaluation(
        &self,
        flag_key: &str,
        resolver_type: &str,
        variant: &str,
        reason: &str,
        duration: std::time::Duration,
    ) {
        let attributes = [
            KeyValue::new("feature_flag.key", flag_key.to_string()),
            KeyValue::new("feature_flag.provider_name", "flagd"),
            KeyValue::new("feature_flag.provider_version", PROVIDER_VERSION),
            KeyValue::new("feature_flag.variant", variant.to_string()),
            KeyValue::new("feature_flag.reason", reason.to_string()),
            KeyValue::new("resolver_type", resolver_type.to_string()),
        ];

        self.evaluation_total.add(1, &attributes);
        self.evaluation_duration
            .record(duration.as_secs_f64(), &attributes);
    }

    /// Record a failed flag evaluation
    pub fn record_evaluation_error(
        &self,
        flag_key: &str,
        resolver_type: &str,
        error_type: &str,
        duration: std::time::Duration,
    ) {
        let attributes = [
            KeyValue::new("feature_flag.key", flag_key.to_string()),
            KeyValue::new("feature_flag.provider_name", "flagd"),
            KeyValue::new("feature_flag.provider_version", PROVIDER_VERSION),
            KeyValue::new("resolver_type", resolver_type.to_string()),
            KeyValue::new("error.type", error_type.to_string()),
        ];

        self.evaluation_total.add(1, &attributes);
        self.evaluation_error_total.add(1, &attributes);
        self.evaluation_duration
            .record(duration.as_secs_f64(), &attributes);
    }
}

impl Default for EvaluationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer for measuring evaluation duration
pub struct EvaluationTimer {
    start: Instant,
}

impl EvaluationTimer {
    /// Start a new evaluation timer
    #[must_use]
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get the elapsed duration
    #[must_use]
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

/// Global metrics instance for convenience
static GLOBAL_METRICS: OnceLock<EvaluationMetrics> = OnceLock::new();

/// Get the global evaluation metrics instance
#[must_use]
pub fn evaluation_metrics() -> &'static EvaluationMetrics {
    GLOBAL_METRICS.get_or_init(EvaluationMetrics::new)
}

/// Record a successful evaluation using global metrics
pub fn record_success(
    flag_key: &str,
    resolver_type: &str,
    variant: &str,
    reason: &str,
    duration: std::time::Duration,
) {
    evaluation_metrics().record_evaluation(flag_key, resolver_type, variant, reason, duration);
}

/// Record a failed evaluation using global metrics
pub fn record_error(
    flag_key: &str,
    resolver_type: &str,
    error_type: &str,
    duration: std::time::Duration,
) {
    evaluation_metrics().record_evaluation_error(flag_key, resolver_type, error_type, duration);
}
