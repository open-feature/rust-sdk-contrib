use std::collections::HashMap;

use mockito::Server;
use open_feature_flipt::flipt::{Config, FliptProvider, NoneAuthentication};
use open_feature_flipt::open_feature::{EvaluationContext, provider::FeatureProvider};

#[tokio::test]
async fn test_boolean() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/evaluate/v1/boolean")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
        "enabled":true,
        "reason":"DEFAULT_EVALUATION_REASON",
        "requestId":"fb502132-66e5-45a1-a315-f1a91d4f4637",
        "requestDurationMillis":3.070422,
        "timestamp":"2024-05-01T10:05:06.822847492Z",
        "flagKey":"flag_boolean"
    }"#,
        )
        .create_async()
        .await;

    let config = Config {
        url: server.url(),
        authentication_strategy: NoneAuthentication::new(),
        timeout: 60,
    };
    let ctx = EvaluationContext {
        targeting_key: None,
        custom_fields: HashMap::new(),
    };

    let provider = FliptProvider::new("default".to_owned(), config).unwrap();
    let details = provider
        .resolve_bool_value("flag_boolean", &ctx)
        .await
        .unwrap();
    assert!(details.value);

    // check if the mock is called once
    mock.assert();
}

#[tokio::test]
async fn test_boolean_unregistered() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/evaluate/v1/boolean")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"code":5,"message":"flag \"default/unregistered\" not found","details":[]}"#)
        .create_async()
        .await;

    let config = Config {
        url: server.url(),
        authentication_strategy: NoneAuthentication::new(),
        timeout: 60,
    };
    let ctx = EvaluationContext {
        targeting_key: None,
        custom_fields: HashMap::new(),
    };

    let provider = FliptProvider::new("default".to_owned(), config).unwrap();
    let res = provider.resolve_bool_value("unregistered", &ctx).await;
    assert!(res.is_err());

    // check if the mock is called once
    mock.assert();
}

#[tokio::test]
async fn test_integer() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/evaluate/v1/variant")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "match":true,
                "segmentKeys":["a"],
                "reason":"MATCH_EVALUATION_REASON",
                "variantKey":"2024",
                "variantAttachment":"",
                "requestId":"da64997c-92ee-4650-9585-cdcba0cb804a",
                "requestDurationMillis":4.353005,
                "timestamp":"2024-05-01T10:38:38.673007435Z",
                "flagKey":"flag_integer"
            }"#,
        )
        .create_async()
        .await;

    let config = Config {
        url: server.url(),
        authentication_strategy: NoneAuthentication::new(),
        timeout: 60,
    };
    let ctx = EvaluationContext {
        targeting_key: None,
        custom_fields: HashMap::new(),
    };

    let provider = FliptProvider::new("default".to_owned(), config).unwrap();
    let details = provider
        .resolve_int_value("flag_integer", &ctx)
        .await
        .unwrap();
    assert!(details.value == 2024);

    // check if the mock is called once
    mock.assert();
}

#[tokio::test]
async fn test_float() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/evaluate/v1/variant")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "match":true,
                "segmentKeys":["a"],
                "reason":"MATCH_EVALUATION_REASON",
                "variantKey":"3.1415",
                "variantAttachment":"",
                "requestId":"da64997c-92ee-4650-9585-cdcba0cb804a",
                "requestDurationMillis":4.353005,
                "timestamp":"2024-05-01T10:38:38.673007435Z",
                "flagKey":"flag_float"
            }"#,
        )
        .create_async()
        .await;

    let config = Config {
        url: server.url(),
        authentication_strategy: NoneAuthentication::new(),
        timeout: 60,
    };
    let ctx = EvaluationContext {
        targeting_key: None,
        custom_fields: HashMap::new(),
    };

    let provider = FliptProvider::new("default".to_owned(), config).unwrap();
    let details = provider
        .resolve_float_value("flag_float", &ctx)
        .await
        .unwrap();
    assert!(3.1 < details.value && details.value < 3.2);

    // check if the mock is called once
    mock.assert();
}

#[tokio::test]
async fn test_string() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/evaluate/v1/variant")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "match":true,
                "segmentKeys":["a"],
                "reason":"MATCH_EVALUATION_REASON",
                "variantKey":"Hello",
                "variantAttachment":"",
                "requestId":"da64997c-92ee-4650-9585-cdcba0cb804a",
                "requestDurationMillis":4.353005,
                "timestamp":"2024-05-01T10:38:38.673007435Z",
                "flagKey":"flag_string"
            }"#,
        )
        .create_async()
        .await;

    let config = Config {
        url: server.url(),
        authentication_strategy: NoneAuthentication::new(),
        timeout: 60,
    };
    let ctx = EvaluationContext {
        targeting_key: None,
        custom_fields: HashMap::new(),
    };

    let provider = FliptProvider::new("default".to_owned(), config).unwrap();
    let details = provider
        .resolve_string_value("flag_string", &ctx)
        .await
        .unwrap();
    assert!(details.value == "Hello");

    // check if the mock is called once
    mock.assert();
}

#[tokio::test]
async fn test_struct() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/evaluate/v1/variant")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "match":true,
                "segmentKeys":["a"],
                "reason":"MATCH_EVALUATION_REASON",
                "variantKey":"a",
                "variantAttachment":"{\"name\":\"Miho Nishizumi\",\"message\":\"Panzer Vor!\",\"age\":17}",
                "requestId":"da64997c-92ee-4650-9585-cdcba0cb804a",
                "requestDurationMillis":4.353005,
                "timestamp":"2024-05-01T10:38:38.673007435Z",
                "flagKey":"flag_struct"
            }"#,
        )
        .create_async()
        .await;

    let config = Config {
        url: server.url(),
        authentication_strategy: NoneAuthentication::new(),
        timeout: 60,
    };
    let ctx = EvaluationContext {
        targeting_key: None,
        custom_fields: HashMap::new(),
    };

    let provider = FliptProvider::new("default".to_owned(), config).unwrap();
    let details = provider
        .resolve_struct_value("flag_struct", &ctx)
        .await
        .unwrap();
    let res = details.value;
    assert!(res.fields.get("name").unwrap().as_str().unwrap() == "Miho Nishizumi");
    assert!(res.fields.get("message").unwrap().as_str().unwrap() == "Panzer Vor!");
    assert!(res.fields.get("age").unwrap().as_i64().unwrap() == 17);

    // check if the mock is called once
    mock.assert();
}
