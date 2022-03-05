mod schema {
    cynic::use_schema!("src/bin/simple.graphql");
}

#[derive(cynic::QueryVariables)]
struct TestArgs {
    an_int: i32,
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(schema_path = "src/bin/simple.graphql", variables = "TestArgs")]
struct TestStruct {
    #[arguments(x: $an_int, y = Some("1".to_string()))]
    field_one: String,
    nested: Nested,
    opt_nested: Option<Nested>,
    dessert: Option<Dessert>,
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(schema_path = "src/bin/simple.graphql", query_module = "schema")]
struct Nested {
    a_string: String,
    opt_string: Option<String>,
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    graphql_type = "Query",
    variables = "TestArgs"
)]
struct TestQuery {
    test_struct: Option<TestStruct>,
}

#[derive(cynic::Enum, Clone, Debug, PartialEq)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub enum Dessert {
    Cheesecake,
    IceCream,
}

fn test_decoding(input: serde_json::Value, expected_result: TestQuery) {
    assert_eq!(
        serde_json::from_value::<TestQuery>(input).unwrap(),
        expected_result
    );
}

#[test]
fn test_decoding_entire_struct() {
    test_decoding(
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
    test_decoding(json, TestQuery { test_struct: None });
}

#[test]
fn test_query_building() {
    use cynic::QueryBuilder;

    let operation = TestQuery::build(TestArgs { an_int: 1 });

    insta::assert_snapshot!(operation.query);
}
