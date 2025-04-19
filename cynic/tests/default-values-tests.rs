//! Tests of the generated serialization code for InputObjects

use serde_json::json;

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

#[test]
fn test_default_arguments_not_required() {
    #[derive(cynic::QueryFragment)]
    #[cynic(graphql_type = "Query", schema_path = "tests/test-schema.graphql")]
    struct Query {
        #[allow(dead_code)]
        field_with_defaults: i32,
    }

    #[derive(cynic::QueryFragment)]
    #[cynic(graphql_type = "Query", schema_path = "tests/test-schema.graphql")]
    struct Query2 {
        #[arguments(anInt: 1, anOptionalInt: null)]
        #[allow(dead_code)]
        field_with_defaults: i32,
    }
}

#[test]
fn test_input_object_skip_serializing_allowed_on_default_fields() {
    #[derive(cynic::InputObject)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct InputWithDefaults {
        #[cynic(skip_serializing_if = "Option::is_none")]
        required_with_default: Option<i32>,
    }

    let without_value = serde_json::to_value(InputWithDefaults {
        required_with_default: None,
    })
    .unwrap();
    assert_eq!(without_value, json!({}));

    let with_value = serde_json::to_value(InputWithDefaults {
        required_with_default: Some(123),
    })
    .unwrap();

    assert_eq!(with_value, json!({ "requiredWithDefault": 123 }));
}

#[test]
fn test_input_object_auth_skip_serializing_on_default_fields() {
    #[derive(cynic::InputObject)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct InputWithDefaults {
        required_with_default: Option<i32>,
    }

    let without_value = serde_json::to_value(InputWithDefaults {
        required_with_default: None,
    })
    .unwrap();
    assert_eq!(without_value, json!({}));

    let with_value = serde_json::to_value(InputWithDefaults {
        required_with_default: Some(123),
    })
    .unwrap();

    assert_eq!(with_value, json!({ "requiredWithDefault": 123 }));
}

#[test]
fn test_default_directive_deserialize() {
    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query", schema_path = "tests/test-schema.graphql")]
    struct Query {
        #[arguments(id: "hello")]
        #[cynic(default)]
        #[allow(dead_code)]
        post: BlogPost,
    }

    #[derive(cynic::QueryFragment, Default, Debug)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct BlogPost {
        #[allow(dead_code)]
        id: Option<cynic::Id>,
    }

    insta::assert_debug_snapshot!(serde_json::from_value::<Query>(json!({"post": null})).unwrap(), @r"
    Query {
        post: BlogPost {
            id: None,
        },
    }
    ");
}
