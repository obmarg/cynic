//! Tests of string escaping in queries
//! Tests of the generated serialization code for InputObjects

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

#[test]
fn test_quotes_in_string() {
    use cynic::QueryBuilder;

    #[derive(cynic::QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Query {
        #[allow(dead_code)]
        #[arguments(input: "Hello \"Graeme\"")]
        field_with_string: i32,
    }

    let query = Query::build(());

    insta::assert_display_snapshot!(query.query, @r###"
    query Query {
      fieldWithString(input: """Hello "Graeme"""")
    }

    "###);
}

#[test]
fn test_newlines_in_string() {
    use cynic::QueryBuilder;

    #[derive(cynic::QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Query {
        #[allow(dead_code)]
        #[arguments(input: "I am a string with \nnew\nlines")]
        field_with_string: i32,
    }

    let query = Query::build(());

    insta::assert_display_snapshot!(query.query, @r###"
    query Query {
      fieldWithString(input: """I am a string with 
      new
      lines""")
    }

    "###);
}

#[test]
fn test_unicode_in_string() {
    use cynic::QueryBuilder;

    #[derive(cynic::QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Query {
        #[allow(dead_code)]
        #[arguments(input: "I am a 😎 string")]
        field_with_string: i32,
    }

    let query = Query::build(());

    insta::assert_display_snapshot!(query.query, @r###"
    query Query {
      fieldWithString(input: """I am a 😎 string""")
    }

    "###);
}
