mod query_dsl {
    type Json = serde_json::Value;

    cynic::query_dsl!("src/bin/simple.graphql");
}

#[derive(cynic::FragmentArguments)]
struct TestArgs {}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "TestStruct",
    //argument_struct = "TestArgs"
)]
struct TestStruct {
    // TODO: Could automatically add Some here, though
    // honestly not sure, as what if the argument itself is some optional in a struct.
    // for now this doesn't seem like the worst decision.
    //#[arguments(x = Some(1), y = Some("1".to_string()))]
    field_one: String,
    nested: Nested,
    opt_nested: Option<Nested>,
    dessert: Option<Dessert>,
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "Nested"
)]
struct Nested {
    a_string: String,
    opt_string: Option<String>,
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "Query",
    argument_struct = "TestArgs"
)]
struct TestQuery {
    test_struct: Option<TestStruct>,
}

#[derive(cynic::Enum, Clone, Debug, PartialEq)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    graphql_type = "Dessert",
    query_module = "query_dsl",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub enum Dessert {
    Cheesecake,
    IceCream,
}

fn run_test(input: serde_json::Value, expected_result: TestQuery) {
    use cynic::{FragmentContext, QueryFragment};

    let query = cynic::Operation::query(TestQuery::fragment(FragmentContext::with_args(
        &TestArgs {},
    )));

    let test_data = cynic::GraphQLResponse {
        errors: None,
        data: Some(input),
    };
    let data = query.decode_response(test_data).unwrap().data.unwrap();
    assert_eq!(data, expected_result);
}

#[test]
fn test_decoding_entire_struct() {
    run_test(
        serde_json::json!({
            "testStruct": {
                "fieldOne": "test",
                "nested": {
                    "aString": "test2",
                    "optString": "test3"
                },
                "optNested": {
                    "aString": "test2",
                    "optString": "test3"
                },
                "dessert": "CHEESECAKE"
            }
        }),
        TestQuery {
            test_struct: Some(TestStruct {
                field_one: "test".to_string(),
                nested: Nested {
                    a_string: "test2".to_string(),
                    opt_string: Some("test3".to_string()),
                },
                opt_nested: Some(Nested {
                    a_string: "test2".to_string(),
                    opt_string: Some("test3".to_string()),
                }),
                dessert: Some(Dessert::Cheesecake),
            }),
        },
    );
}

#[test]
fn test_decoding_options() {
    let json = serde_json::json!({ "testStruct": null });
    run_test(json, TestQuery { test_struct: None });
}
