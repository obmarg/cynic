use serde_json::json;

#[derive(cynic::QueryVariables, cynic::QueryVariableLiterals)]
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
#[cynic(schema_path = "../schemas/simple.graphql", variables = "TestArgs")]
struct TestStruct {
    #[arguments(x: 1, y: "hello")]
    field_one: String,
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(
    schema_path = "../schemas/simple.graphql",
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

mod variable_inlining {
    use cynic::OperationBuilder;

    use super::{TestArgs, TestArgsFields, schema};

    #[derive(cynic::QueryFragment, PartialEq, Debug)]
    #[cynic(schema_path = "../schemas/simple.graphql", variables = "TestArgs")]
    struct TestStruct {
        #[arguments(x: 1, y: $a_str)]
        field_one: String,
    }

    #[derive(cynic::QueryFragment, PartialEq, Debug)]
    #[cynic(
        schema_path = "../schemas/simple.graphql",
        graphql_type = "Query",
        variables = "TestArgs"
    )]
    struct QueryWithArguments {
        test_struct: Option<TestStruct>,
    }

    #[test]
    fn test_variable_inlining() {
        let operation = OperationBuilder::<QueryWithArguments, TestArgs<'_>>::query()
            .with_variables(TestArgs {
                a_str: Some("boom, this is interpolated"),
            })
            .build_with_variables_inlined()
            .unwrap();

        insta::assert_display_snapshot!(operation.query, @r###"
        query QueryWithArguments {
          testStruct {
            fieldOne(x: 1, y: "boom, this is interpolated")
          }
        }
        "###);
    }

    #[test]
    fn test_skip_serializing_if_none_for_inlines() {
        let operation = OperationBuilder::<QueryWithArguments, TestArgs<'_>>::query()
            .with_variables(TestArgs { a_str: None })
            .build_with_variables_inlined()
            .unwrap();

        insta::assert_display_snapshot!(operation.query, @r###"
        query QueryWithArguments {
          testStruct {
            fieldOne(x: 1)
          }
        }
        "###);
    }
}

mod schema {
    cynic::use_schema!("../schemas/simple.graphql");
}
