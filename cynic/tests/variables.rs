use serde_json::json;

#[derive(cynic::QueryVariables)]
struct TestArgs<'a> {
    #[cynic(skip_serializing_if = "Option::is_none")]
    a_str: Option<&'a str>,
}

#[test]
fn test_skip_serializing_if_none() {
    assert_eq!(
        serde_json::to_value(TestArgs { a_str: None }).unwrap(),
        json!({})
    );
    assert_eq!(
        serde_json::to_value(TestArgs {
            a_str: Some("hello")
        })
        .unwrap(),
        json!({"aStr": "hello"})
    );
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(schema_path = "src/bin/simple.graphql", variables = "TestArgs")]
struct TestStruct {
    #[arguments(x: 1, y: "hello")]
    field_one: String,
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    graphql_type = "Query",
    variables = "TestArgs"
)]
struct QueryWithUnusedVariables {
    test_struct: Option<TestStruct>,
}

#[test]
fn test_unused_variables_not_rendered() {
    use cynic::QueryBuilder;

    let operation = QueryWithUnusedVariables::build(TestArgs { a_str: None });

    insta::assert_snapshot!(operation.query, @r###"
    query QueryWithUnusedVariables {
      testStruct {
        fieldOne(x: 1, y: "hello")
      }
    }

    "###);
}

mod schema {
    cynic::use_schema!("src/bin/simple.graphql");
}
