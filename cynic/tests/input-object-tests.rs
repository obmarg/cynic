//! Tests of the generated serialization code for InputObjects

use serde_json::json;

mod query_dsl {
    cynic::query_dsl!("tests/test-schema.graphql");
}

#[test]
fn test_input_object_renames() {
    #![allow(non_snake_case)]

    use cynic::SerializableArgument;

    #[derive(cynic::InputObject)]
    #[cynic(
        graphql_type = "BlogPostInput",
        schema_path = "tests/test-schema.graphql",
        query_module = "query_dsl",
        rename_all = "lowercase"
    )]
    struct BlogPost {
        Content: String,
        #[cynic(rename = "author")]
        writer: Option<String>,
    }

    let post = BlogPost {
        Content: "hi".into(),
        writer: Some("Me".into()),
    }
    .serialize()
    .unwrap();

    assert_eq!(post, json!({ "content": "hi", "author": "Me" }));
}

#[test]
fn test_input_object_skip_serializing() {
    use cynic::SerializableArgument;

    #[derive(cynic::InputObject)]
    #[cynic(
        graphql_type = "BlogPostInput",
        schema_path = "tests/test-schema.graphql",
        query_module = "query_dsl"
    )]
    struct BlogPost {
        content: String,
        #[cynic(skip_serializing_if = "Option::is_none")]
        author: Option<String>,
    }

    let without_author = BlogPost {
        content: "hi".into(),
        author: None,
    }
    .serialize()
    .unwrap();
    assert_eq!(without_author, json!({ "content": "hi" }));

    let with_author = BlogPost {
        content: "hi".into(),
        author: Some("Me".into()),
    }
    .serialize()
    .unwrap();
    assert_eq!(with_author, json!({ "content": "hi", "author": "Me" }));
}
