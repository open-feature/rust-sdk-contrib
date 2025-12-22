//! OpenTelemetry instrumentation for flagd provider
//!
//! This module provides tracing, metrics, and context propagation for all flagd resolvers:
//! - RPC (gRPC) resolver
//! - REST (HTTP/OFREP) resolver  
//! - In-process resolver
//!
//! Enable with the `otel` cargo feature.

#[cfg(any(feature = "rpc", feature = "in-process"))]
pub mod grpc;

#[cfg(feature = "rest")]
pub mod http;

pub mod metrics;
mod propagation;
mod span;

#[cfg(test)]
mod tests;

pub use metrics::{
    EvaluationMetrics, EvaluationTimer, evaluation_metrics, record_error, record_success,
};
pub use propagation::*;
pub use span::*;
