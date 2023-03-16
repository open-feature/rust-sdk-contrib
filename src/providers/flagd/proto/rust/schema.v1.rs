// @generated
/// Request body for bulk flag evaluation, used by the ResolveAll rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveAllRequest {
    /// Object structure describing the EvaluationContext used in the flag evaluation, see <https://docs.openfeature.dev/docs/reference/concepts/evaluation-context>
    #[prost(message, optional, tag="1")]
    pub context: ::core::option::Option<::prost_types::Struct>,
}
/// Response body for bulk flag evaluation, used by the ResolveAll rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveAllResponse {
    /// Object structure describing the evaluated flags for the provided context.
    #[prost(map="string, message", tag="1")]
    pub flags: ::std::collections::HashMap<::prost::alloc::string::String, AnyFlag>,
}
/// A variant type flag response.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnyFlag {
    /// The reason for the given return value, see <https://docs.openfeature.dev/docs/specification/types#resolution-details>
    #[prost(string, tag="1")]
    pub reason: ::prost::alloc::string::String,
    /// The variant name of the returned flag value.
    #[prost(string, tag="2")]
    pub variant: ::prost::alloc::string::String,
    /// The response value of the boolean flag evaluation, will be unset in the case of error.
    #[prost(oneof="any_flag::Value", tags="3, 4, 5, 6")]
    pub value: ::core::option::Option<any_flag::Value>,
}
/// Nested message and enum types in `AnyFlag`.
pub mod any_flag {
    /// The response value of the boolean flag evaluation, will be unset in the case of error.
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(bool, tag="3")]
        BoolValue(bool),
        #[prost(string, tag="4")]
        StringValue(::prost::alloc::string::String),
        #[prost(double, tag="5")]
        DoubleValue(f64),
        #[prost(message, tag="6")]
        ObjectValue(::prost_types::Struct),
    }
}
/// Request body for boolean flag evaluation, used by the ResolveBoolean rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveBooleanRequest {
    /// Flag key of the requested flag.
    #[prost(string, tag="1")]
    pub flag_key: ::prost::alloc::string::String,
    /// Object structure describing the EvaluationContext used in the flag evaluation, see <https://docs.openfeature.dev/docs/reference/concepts/evaluation-context>
    #[prost(message, optional, tag="2")]
    pub context: ::core::option::Option<::prost_types::Struct>,
}
/// Response body for boolean flag evaluation. used by the ResolveBoolean rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveBooleanResponse {
    /// The response value of the boolean flag evaluation, will be unset in the case of error.
    #[prost(bool, tag="1")]
    pub value: bool,
    /// The reason for the given return value, see <https://docs.openfeature.dev/docs/specification/types#resolution-details>
    #[prost(string, tag="2")]
    pub reason: ::prost::alloc::string::String,
    /// The variant name of the returned flag value.
    #[prost(string, tag="3")]
    pub variant: ::prost::alloc::string::String,
}
/// Request body for string flag evaluation, used by the ResolveString rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveStringRequest {
    /// Flag key of the requested flag.
    #[prost(string, tag="1")]
    pub flag_key: ::prost::alloc::string::String,
    /// Object structure describing the EvaluationContext used in the flag evaluation, see <https://docs.openfeature.dev/docs/reference/concepts/evaluation-context>
    #[prost(message, optional, tag="2")]
    pub context: ::core::option::Option<::prost_types::Struct>,
}
/// Response body for string flag evaluation. used by the ResolveString rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveStringResponse {
    /// The response value of the string flag evaluation, will be unset in the case of error.
    #[prost(string, tag="1")]
    pub value: ::prost::alloc::string::String,
    /// The reason for the given return value, see <https://docs.openfeature.dev/docs/specification/types#resolution-details>
    #[prost(string, tag="2")]
    pub reason: ::prost::alloc::string::String,
    /// The variant name of the returned flag value.
    #[prost(string, tag="3")]
    pub variant: ::prost::alloc::string::String,
}
/// Request body for float flag evaluation, used by the ResolveFloat rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveFloatRequest {
    /// Flag key of the requested flag.
    #[prost(string, tag="1")]
    pub flag_key: ::prost::alloc::string::String,
    /// Object structure describing the EvaluationContext used in the flag evaluation, see <https://docs.openfeature.dev/docs/reference/concepts/evaluation-context>
    #[prost(message, optional, tag="2")]
    pub context: ::core::option::Option<::prost_types::Struct>,
}
/// Response body for float flag evaluation. used by the ResolveFloat rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveFloatResponse {
    /// The response value of the float flag evaluation, will be empty in the case of error.
    #[prost(double, tag="1")]
    pub value: f64,
    /// The reason for the given return value, see <https://docs.openfeature.dev/docs/specification/types#resolution-details>
    #[prost(string, tag="2")]
    pub reason: ::prost::alloc::string::String,
    /// The variant name of the returned flag value.
    #[prost(string, tag="3")]
    pub variant: ::prost::alloc::string::String,
}
/// Request body for int flag evaluation, used by the ResolveInt rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveIntRequest {
    /// Flag key of the requested flag.
    #[prost(string, tag="1")]
    pub flag_key: ::prost::alloc::string::String,
    /// Object structure describing the EvaluationContext used in the flag evaluation, see <https://docs.openfeature.dev/docs/reference/concepts/evaluation-context>
    #[prost(message, optional, tag="2")]
    pub context: ::core::option::Option<::prost_types::Struct>,
}
/// Response body for int flag evaluation. used by the ResolveInt rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveIntResponse {
    /// The response value of the int flag evaluation, will be unset in the case of error.
    #[prost(int64, tag="1")]
    pub value: i64,
    /// The reason for the given return value, see <https://docs.openfeature.dev/docs/specification/types#resolution-details>
    #[prost(string, tag="2")]
    pub reason: ::prost::alloc::string::String,
    /// The variant name of the returned flag value.
    #[prost(string, tag="3")]
    pub variant: ::prost::alloc::string::String,
}
/// Request body for object flag evaluation, used by the ResolveObject rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveObjectRequest {
    /// Flag key of the requested flag.
    #[prost(string, tag="1")]
    pub flag_key: ::prost::alloc::string::String,
    /// Object structure describing the EvaluationContext used in the flag evaluation, see <https://docs.openfeature.dev/docs/reference/concepts/evaluation-context>
    #[prost(message, optional, tag="2")]
    pub context: ::core::option::Option<::prost_types::Struct>,
}
/// Response body for object flag evaluation. used by the ResolveObject rpc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResolveObjectResponse {
    /// The response value of the object flag evaluation, will be unset in the case of error.
    ///
    /// NOTE: This structure will need to be decoded from google/protobuf/struct.proto before it is returned to the SDK
    #[prost(message, optional, tag="1")]
    pub value: ::core::option::Option<::prost_types::Struct>,
    /// The reason for the given return value, see <https://docs.openfeature.dev/docs/specification/types#resolution-details>
    #[prost(string, tag="2")]
    pub reason: ::prost::alloc::string::String,
    /// The variant name of the returned flag value.
    #[prost(string, tag="3")]
    pub variant: ::prost::alloc::string::String,
}
/// Response body for the EventStream stream response
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventStreamResponse {
    /// String key indicating the type of event that is being received, for example, provider_ready or configuration_change
    #[prost(string, tag="1")]
    pub r#type: ::prost::alloc::string::String,
    /// Object structure for use when sending relevant metadata to provide context to the event.
    /// Can be left unset when it is not required.
    #[prost(message, optional, tag="2")]
    pub data: ::core::option::Option<::prost_types::Struct>,
}
/// Empty stream request body
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventStreamRequest {
}
include!("schema.v1.serde.rs");
include!("schema.v1.tonic.rs");
// @@protoc_insertion_point(module)