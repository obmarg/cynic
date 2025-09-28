//! Tests of the generated serialization code for InputObjects

use serde_json::json;

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

#[test]
fn test_input_object_renames() {
    #![allow(non_snake_case)]

    #[derive(cynic::InputObject)]
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
    #[derive(cynic::InputObject)]
    #[cynic(
        graphql_type = "BlogPostInput",
        schema_path = "tests/test-schema.graphql"
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
    #[derive(cynic::InputObject)]
    #[cynic(
        graphql_type = "BlogPostInput",
        schema_path = "tests/test-schema.graphql"
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
    content: hi
    author: me
    "###
    );
}

#[test]
fn test_oneof_objects_as_enum() {
    #[derive(cynic::InputObject)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct InputWithDefaults {
        optional_int: i32,
    }

    #[derive(cynic::InputObject)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    enum OneOfInput {
        Foo(String),
        Bar(i32),
        Nested(InputWithDefaults),
    }

    insta::assert_json_snapshot!(vec![
            OneOfInput::Foo("hello".into()),
            OneOfInput::Bar(123),
            OneOfInput::Nested(InputWithDefaults { optional_int: 1 })
        ],
        @r#"
    [
      {
        "foo": "hello"
      },
      {
        "bar": 123
      },
      {
        "nested": {
          "optionalInt": 1
        }
      }
    ]
    "#
    )
}

#[test]
fn test_oneof_objects_as_struct() {
    #[derive(cynic::InputObject)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct OneOfInput<'a> {
        foo: Option<&'a str>,
        bar: Option<i32>,
    }

    insta::assert_json_snapshot!(vec![
            OneOfInput { foo: Some("hello"), bar: None },
            OneOfInput { foo: None, bar: Some(123),  }
        ],
        @r#"
    [
      {
        "foo": "hello"
      },
      {
        "bar": 123
      }
    ]
    "#
    )
}
