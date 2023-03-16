// @generated
/// SyncFlagsRequest is the request initiating the sever-streaming rpc. Flagd sends this request, acting as the client
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyncFlagsRequest {
    /// Optional: A unique identifier for flagd provider (grpc client) initiating the request. The server implementations
    /// can utilize this identifier to aggregate flag configurations and stream them to a specific client. This identifier
    /// is intended to be optional. However server implementation may enforce it.
    #[prost(string, tag="1")]
    pub provider_id: ::prost::alloc::string::String,
}
/// SyncFlagsResponse is the server response containing feature flag configurations and the state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyncFlagsResponse {
    /// flagd feature flag configuration. Must be validated to schema - <https://raw.githubusercontent.com/open-feature/schemas/main/json/flagd-definitions.json>
    #[prost(string, tag="1")]
    pub flag_configuration: ::prost::alloc::string::String,
    /// State conveying the operation to be performed by flagd. See the descriptions of SyncState for an explanation of
    /// supported values
    #[prost(enumeration="SyncState", tag="2")]
    pub state: i32,
}
/// FetchAllFlagsRequest is the request to fetch all flags. Flagd sends this request as the client in order to resync its internal state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FetchAllFlagsRequest {
    /// Optional: A unique identifier for flagd provider (grpc client) initiating the request. The server implementations
    /// can utilize this identifier to aggregate flag configurations and stream them to a specific client. This identifier
    /// is intended to be optional. However server implementation may enforce it.
    #[prost(string, tag="1")]
    pub provider_id: ::prost::alloc::string::String,
}
///   FetchAllFlagsResponse is the server response containing feature flag configurations
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FetchAllFlagsResponse {
    /// flagd feature flag configuration. Must be validated to schema - <https://raw.githubusercontent.com/open-feature/schemas/main/json/flagd-definitions.json>
    #[prost(string, tag="1")]
    pub flag_configuration: ::prost::alloc::string::String,
}
/// SyncState conveys the state of the payload. These states are related to flagd isync.go type definitions but
/// contains extras to optimize grpc use case. Refer - <https://github.com/open-feature/flagd/blob/main/pkg/sync/isync.go>
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SyncState {
    /// Value is ignored by the listening flagd
    Unspecified = 0,
    /// All the flags matching the request. This is the default response and other states can be ignored
    /// by the implementation. Flagd internally replaces all existing flags for this response state.
    All = 1,
    /// Convey an addition of a flag. Flagd internally handles this by combining new flags with existing ones
    Add = 2,
    /// Convey an update of a flag. Flagd internally attempts to update if the updated flag already exist OR if it does not,
    /// it will get added
    Update = 3,
    /// Convey a deletion of a flag. Flagd internally removes the flag
    Delete = 4,
    /// Optional server ping to check client connectivity. Handling is ignored by flagd and is to merely support live check
    Ping = 5,
}
impl SyncState {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SyncState::Unspecified => "SYNC_STATE_UNSPECIFIED",
            SyncState::All => "SYNC_STATE_ALL",
            SyncState::Add => "SYNC_STATE_ADD",
            SyncState::Update => "SYNC_STATE_UPDATE",
            SyncState::Delete => "SYNC_STATE_DELETE",
            SyncState::Ping => "SYNC_STATE_PING",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SYNC_STATE_UNSPECIFIED" => Some(Self::Unspecified),
            "SYNC_STATE_ALL" => Some(Self::All),
            "SYNC_STATE_ADD" => Some(Self::Add),
            "SYNC_STATE_UPDATE" => Some(Self::Update),
            "SYNC_STATE_DELETE" => Some(Self::Delete),
            "SYNC_STATE_PING" => Some(Self::Ping),
            _ => None,
        }
    }
}
include!("sync.v1.serde.rs");
include!("sync.v1.tonic.rs");
// @@protoc_insertion_point(module)