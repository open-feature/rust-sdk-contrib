//! Flagd Evaluation Engine
//!
//! A JSONLogic-based evaluation engine for flagd, providing local flag evaluation
//! with support for targeting rules, fractional rollouts, and semantic version comparisons.

pub mod error;
pub mod model;
pub mod targeting;

// Re-export main types for ease of use
pub use error::FlagdEvaluationError;
pub use model::{FeatureFlag, FlagParser};
pub use targeting::Operator;
