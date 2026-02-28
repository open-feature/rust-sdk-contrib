// Re-export from flagd-evaluation-engine crate
pub use flagd_evaluation_engine::model;
pub use flagd_evaluation_engine::targeting;

pub use flagd_evaluation_engine::Operator;
pub use flagd_evaluation_engine::error::FlagdEvaluationError as FlagdError;
pub use flagd_evaluation_engine::{FeatureFlag, FlagParser};

// Storage module (async storage - specific to flagd)
pub mod storage;
pub use storage::{
    Connector, FileConnector, FlagStore, GrpcStreamConnector, QueuePayload, QueuePayloadType,
    StorageQueryResult, StorageState, StorageStateChange,
};

// Resolver modules (specific to flagd, not extracted)
pub mod resolver;
