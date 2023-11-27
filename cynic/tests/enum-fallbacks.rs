use cynic::Enum;
use serde::Deserialize;
use serde_json::json;

#[derive(Enum, Debug, PartialEq)]
#[cynic(graphql_type = "PostState", schema_path = "tests/test-schema.graphql")]
enum PostStateEmptyFallback {
    Posted,
    Draft,
    #[cynic(fallback)]
    Unknown,
}

#[derive(Enum, Debug, PartialEq)]
#[cynic(graphql_type = "PostState", schema_path = "tests/test-schema.graphql")]
enum PostStateStringFallback {
    Posted,
    Draft,
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(Enum, Debug, PartialEq)]
#[cynic(graphql_type = "PostState", schema_path = "tests/test-schema.graphql")]
enum PostStateNonexhaustiveFallback {
    Posted,
    #[cynic(fallback)]
    Unknown(String),
}

#[allow(non_snake_case, non_camel_case_types)]
mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

#[test]
fn test_empty_fallback() {
    assert_eq!(
        PostStateEmptyFallback::deserialize(json!("BLAH")).unwrap(),
        PostStateEmptyFallback::Unknown
    );

    serde_json::to_value(PostStateEmptyFallback::Unknown).expect_err("should fail");
}

#[test]
fn test_string_fallback() {
    assert_eq!(
        PostStateStringFallback::deserialize(json!("BLAH")).unwrap(),
        PostStateStringFallback::Unknown("BLAH".to_string())
    );

    let val = serde_json::to_value(PostStateStringFallback::Unknown("BLAH".to_string())).unwrap();

    assert_eq!(val, serde_json::Value::String("BLAH".into()));
}

#[test]
fn test_nonexhaustive_fallback() {
    assert_eq!(
        PostStateNonexhaustiveFallback::deserialize(json!("DRAFT")).unwrap(),
        PostStateNonexhaustiveFallback::Unknown("DRAFT".to_string())
    );

    let val =
        serde_json::to_value(PostStateNonexhaustiveFallback::Unknown("DRAFT".to_string())).unwrap();

    assert_eq!(val, serde_json::Value::String("DRAFT".into()));
}
