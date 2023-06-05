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

    let operation = Query::build(());

    graphql_parser::parse_query::<'_, String>(&operation.query).expect("a parsable query");

    insta::assert_display_snapshot!(operation.query, @r###"
    query Query {
      fieldWithString(input: "Hello \"Graeme\"")
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
        #[arguments(input: "I am a string with \nnew\nlines\r\tand tabs")]
        field_with_string: i32,
    }

    let operation = Query::build(());

    graphql_parser::parse_query::<'_, String>(&operation.query).expect("a parsable query");

    insta::assert_display_snapshot!(operation.query, @r###"
    query Query {
      fieldWithString(input: "I am a string with \nnew\nlines\r\tand tabs")
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

    let operation = Query::build(());

    graphql_parser::parse_query::<'_, String>(&operation.query).expect("a parsable query");

    insta::assert_display_snapshot!(operation.query, @r###"
    query Query {
      fieldWithString(input: "I am a 😎 string")
    }

    "###);
}

#[test]
fn test_adverserial_block_strings() {
    use cynic::QueryBuilder;

    #[derive(cynic::QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Query {
        #[allow(dead_code)]
        #[arguments(input: r#"I am a 😎 string""#)]
        field_with_string: i32,
    }

    let operation = Query::build(());

    graphql_parser::parse_query::<'_, String>(&operation.query).expect("a parsable query");

    insta::assert_display_snapshot!(operation.query, @r###"
    query Query {
      fieldWithString(input: "I am a 😎 string\"")
    }

    "###);
}
