//! Tests of the generated serialization code for InputObjects

use serde_json::json;

mod schema {
    cynic::use_schema_2!("tests/test-schema.graphql");
}

#[test]
fn test_input_object_renames() {
    #![allow(non_snake_case)]

    #[derive(cynic::InputObject2)]
    #[cynic(
        graphql_type = "BlogPostInput",
        schema_path = "tests/test-schema.graphql",
        rename_all = "lowercase"
    )]
    struct BlogPost {
        Content: String,
        #[cynic(rename = "author")]
        writer: Option<String>,
    }

    let post = serde_json::to_value(BlogPost {
        Content: "hi".into(),
        writer: Some("Me".into()),
    })
    .unwrap();

    assert_eq!(post, json!({ "content": "hi", "author": "Me" }));
}

#[test]
fn test_input_object_skip_serializing() {
    #[derive(cynic::InputObject2)]
    #[cynic(
        graphql_type = "BlogPostInput",
        schema_path = "tests/test-schema.graphql",
        query_module = "schema"
    )]
    struct BlogPost {
        content: String,
        #[cynic(skip_serializing_if = "Option::is_none")]
        author: Option<String>,
    }

    let without_author = serde_json::to_value(BlogPost {
        content: "hi".into(),
        author: None,
    })
    .unwrap();
    assert_eq!(without_author, json!({ "content": "hi" }));

    let with_author = serde_json::to_value(BlogPost {
        content: "hi".into(),
        author: Some("Me".into()),
    })
    .unwrap();

    assert_eq!(with_author, json!({ "content": "hi", "author": "Me" }));
}

#[test]
fn test_input_object_stable_order() {
    #[derive(cynic::InputObject2)]
    #[cynic(
        graphql_type = "BlogPostInput",
        schema_path = "tests/test-schema.graphql",
        query_module = "schema"
    )]
    struct BlogPost {
        content: String,
        #[cynic(skip_serializing_if = "Option::is_none")]
        author: Option<String>,
    }

    // Using a snapshot to ensure we have a stable order
    insta::assert_yaml_snapshot!(
        BlogPost {
            content: "hi".into(),
            author: Some("me".into()),
        },
        @r###"
    ---
    author: me
    content: hi
    "###
    );
}
