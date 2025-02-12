// use common::{Envoy, Flagd, ENVOY_CONFIG, FLAGD_CONFIG, FLAGD_OFREP_PORT, FLAGD_PORT, FLAGD_SYNC_PORT};
// use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
// use open_feature::provider::FeatureProvider;
// use open_feature::EvaluationContext;
// use testcontainers::runners::AsyncRunner;

// mod common;

// #[tokio::test]
// async fn test_envoy_name_resolver() {
//     tracing_subscriber::fmt::init();
//     let flagd = Flagd::new()
//         .with_config(FLAGD_CONFIG)
//         .start()
//         .await
//         .unwrap();

//     let flagd_name = flagd.id();  // Remove leading slash from container name

//     let envoy = Envoy::new()
//         .with_config(&format!("{}", ENVOY_CONFIG.replace("flagd_host", flagd_name)))
//         .start()
//         .await
//         .unwrap();
//     let envoy_port = envoy.get_host_port_ipv4(9211).await.unwrap();

//     // Add delay to ensure Envoy is ready
//     tokio::time::sleep(std::time::Duration::from_secs(2)).await;

//     let provider = FlagdProvider::new(FlagdOptions {
//         host: "localhost".to_string(),
//         port: envoy_port,
//         target_uri: Some(format!("envoy://localhost:{}/test.service", envoy_port)),
//         resolver_type: ResolverType::InProcess,
//         deadline_ms: 5000,
//         ..Default::default()
//     })
//     .await
//     .unwrap();

//     let context = EvaluationContext::default().with_targeting_key("test-user");
    
//     let bool_result = provider.resolve_bool_value("bool-flag", &context)
//         .await
//         .unwrap();
//     assert_eq!(bool_result.value, true);

//     let string_result = provider.resolve_string_value("string-flag", &context)
//         .await
//         .unwrap();
//     assert_eq!(string_result.value, "hello");
// }

// #[tokio::test]
// async fn test_envoy_name_resolver_rest() {
//     pub const ENVOY_REST_CONFIG: &str = r#"
//     static_resources:
//       listeners:
//       - name: listener_0
//         address:
//           socket_address:
//             address: 0.0.0.0
//             port_value: 9211
//         filter_chains:
//         - filters:
//           - name: envoy.filters.network.http_connection_manager
//             typed_config:
//               "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
//               stat_prefix: ingress_http
//               route_config:
//                 name: local_route
//                 virtual_hosts:
//                 - name: local_service
//                   domains: ["*"]
//                   routes:
//                   - match:
//                       prefix: "/"
//                     route:
//                       cluster: flagd_service
//               http_filters:
//               - name: envoy.filters.http.router
//                 typed_config:
//                   "@type": type.googleapis.com/envoy.extensions.filters.http.router.v3.Router
//       clusters:
//       - name: flagd_service
//         connect_timeout: 1s
//         type: STRICT_DNS
//         lb_policy: ROUND_ROBIN
//         load_assignment:
//           cluster_name: flagd_service
//           endpoints:
//           - lb_endpoints:
//             - endpoint:
//                 address:
//                   socket_address:
//                     address: flagd
//                     port_value: 8016
//     "#;
    


//     tracing_subscriber::fmt::init();
//     let flagd = Flagd::new()
//         .with_config(FLAGD_CONFIG)
//         .start()
//         .await
//         .unwrap();

//     let flagd_ofrep_port = flagd.get_host_port_ipv4(FLAGD_OFREP_PORT).await.unwrap();

//     let envoy = Envoy::new()
//         .with_config(&format!("{}", ENVOY_REST_CONFIG.replace("8016", &flagd_ofrep_port.to_string())))
//         .start()
//         .await
//         .unwrap();
//     let envoy_port = envoy.get_host_port_ipv4(9211).await.unwrap();

//     // Add delay to ensure Envoy is ready
//     tokio::time::sleep(std::time::Duration::from_secs(2)).await;

//     let provider = FlagdProvider::new(FlagdOptions {
//         host: "localhost".to_string(),
//         port: envoy_port,
//         target_uri: Some(format!("envoy://localhost:{}/test.service", envoy_port)),
//         resolver_type: ResolverType::Rest,
//         deadline_ms: 5000,
//         ..Default::default()
//     })
//     .await
//     .unwrap();

//     let context = EvaluationContext::default().with_targeting_key("test-user");
    
//     let bool_result = provider.resolve_bool_value("bool-flag", &context)
//         .await
//         .unwrap();
//     assert_eq!(bool_result.value, true);

//     let string_result = provider.resolve_string_value("string-flag", &context)
//         .await
//         .unwrap();
//     assert_eq!(string_result.value, "hello");
// }