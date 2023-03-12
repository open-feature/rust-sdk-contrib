// @generated
/// Generated client implementations.
pub mod service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /** Service defines the exposed rpcs of flagd
*/
    #[derive(Debug, Clone)]
    pub struct ServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            ServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        ///
        pub async fn resolve_all(
            &mut self,
            request: impl tonic::IntoRequest<super::ResolveAllRequest>,
        ) -> Result<tonic::Response<super::ResolveAllResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema.v1.Service/ResolveAll",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn resolve_boolean(
            &mut self,
            request: impl tonic::IntoRequest<super::ResolveBooleanRequest>,
        ) -> Result<tonic::Response<super::ResolveBooleanResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema.v1.Service/ResolveBoolean",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn resolve_string(
            &mut self,
            request: impl tonic::IntoRequest<super::ResolveStringRequest>,
        ) -> Result<tonic::Response<super::ResolveStringResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema.v1.Service/ResolveString",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn resolve_float(
            &mut self,
            request: impl tonic::IntoRequest<super::ResolveFloatRequest>,
        ) -> Result<tonic::Response<super::ResolveFloatResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema.v1.Service/ResolveFloat",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn resolve_int(
            &mut self,
            request: impl tonic::IntoRequest<super::ResolveIntRequest>,
        ) -> Result<tonic::Response<super::ResolveIntResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema.v1.Service/ResolveInt",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn resolve_object(
            &mut self,
            request: impl tonic::IntoRequest<super::ResolveObjectRequest>,
        ) -> Result<tonic::Response<super::ResolveObjectResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema.v1.Service/ResolveObject",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        ///
        pub async fn event_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::EventStreamRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::EventStreamResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema.v1.Service/EventStream",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ServiceServer.
    #[async_trait]
    pub trait Service: Send + Sync + 'static {
        ///
        async fn resolve_all(
            &self,
            request: tonic::Request<super::ResolveAllRequest>,
        ) -> Result<tonic::Response<super::ResolveAllResponse>, tonic::Status>;
        ///
        async fn resolve_boolean(
            &self,
            request: tonic::Request<super::ResolveBooleanRequest>,
        ) -> Result<tonic::Response<super::ResolveBooleanResponse>, tonic::Status>;
        ///
        async fn resolve_string(
            &self,
            request: tonic::Request<super::ResolveStringRequest>,
        ) -> Result<tonic::Response<super::ResolveStringResponse>, tonic::Status>;
        ///
        async fn resolve_float(
            &self,
            request: tonic::Request<super::ResolveFloatRequest>,
        ) -> Result<tonic::Response<super::ResolveFloatResponse>, tonic::Status>;
        ///
        async fn resolve_int(
            &self,
            request: tonic::Request<super::ResolveIntRequest>,
        ) -> Result<tonic::Response<super::ResolveIntResponse>, tonic::Status>;
        ///
        async fn resolve_object(
            &self,
            request: tonic::Request<super::ResolveObjectRequest>,
        ) -> Result<tonic::Response<super::ResolveObjectResponse>, tonic::Status>;
        /// Server streaming response type for the EventStream method.
        type EventStreamStream: futures_core::Stream<
                Item = Result<super::EventStreamResponse, tonic::Status>,
            >
            + Send
            + 'static;
        ///
        async fn event_stream(
            &self,
            request: tonic::Request<super::EventStreamRequest>,
        ) -> Result<tonic::Response<Self::EventStreamStream>, tonic::Status>;
    }
    /** Service defines the exposed rpcs of flagd
*/
    #[derive(Debug)]
    pub struct ServiceServer<T: Service> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Service> ServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ServiceServer<T>
    where
        T: Service,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/schema.v1.Service/ResolveAll" => {
                    #[allow(non_camel_case_types)]
                    struct ResolveAllSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::ResolveAllRequest>
                    for ResolveAllSvc<T> {
                        type Response = super::ResolveAllResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ResolveAllRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).resolve_all(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ResolveAllSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema.v1.Service/ResolveBoolean" => {
                    #[allow(non_camel_case_types)]
                    struct ResolveBooleanSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::ResolveBooleanRequest>
                    for ResolveBooleanSvc<T> {
                        type Response = super::ResolveBooleanResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ResolveBooleanRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).resolve_boolean(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ResolveBooleanSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema.v1.Service/ResolveString" => {
                    #[allow(non_camel_case_types)]
                    struct ResolveStringSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::ResolveStringRequest>
                    for ResolveStringSvc<T> {
                        type Response = super::ResolveStringResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ResolveStringRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).resolve_string(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ResolveStringSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema.v1.Service/ResolveFloat" => {
                    #[allow(non_camel_case_types)]
                    struct ResolveFloatSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::ResolveFloatRequest>
                    for ResolveFloatSvc<T> {
                        type Response = super::ResolveFloatResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ResolveFloatRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).resolve_float(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ResolveFloatSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema.v1.Service/ResolveInt" => {
                    #[allow(non_camel_case_types)]
                    struct ResolveIntSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::ResolveIntRequest>
                    for ResolveIntSvc<T> {
                        type Response = super::ResolveIntResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ResolveIntRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).resolve_int(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ResolveIntSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema.v1.Service/ResolveObject" => {
                    #[allow(non_camel_case_types)]
                    struct ResolveObjectSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::ResolveObjectRequest>
                    for ResolveObjectSvc<T> {
                        type Response = super::ResolveObjectResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ResolveObjectRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).resolve_object(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ResolveObjectSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema.v1.Service/EventStream" => {
                    #[allow(non_camel_case_types)]
                    struct EventStreamSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::ServerStreamingService<super::EventStreamRequest>
                    for EventStreamSvc<T> {
                        type Response = super::EventStreamResponse;
                        type ResponseStream = T::EventStreamStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EventStreamRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).event_stream(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EventStreamSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Service> Clone for ServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Service> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Service> tonic::server::NamedService for ServiceServer<T> {
        const NAME: &'static str = "schema.v1.Service";
    }
}
