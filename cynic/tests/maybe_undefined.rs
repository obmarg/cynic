use cynic::MaybeUndefined;

mod schema {
    cynic::use_schema!("../schemas/simple.graphql");
}

#[derive(cynic::QueryVariables)]
struct TestArgs<'a> {
    #[cynic(skip_serializing_if = "MaybeUndefined::is_undefined")]
    a_str: MaybeUndefined<&'a str>,
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(schema_path = "../schemas/simple.graphql", variables = "TestArgs")]
struct TestStruct {
    field_one: String,
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(
    schema_path = "../schemas/simple.graphql",
    graphql_type = "Query",
    variables = "TestArgs"
)]
struct TestQuery {
    test_struct: MaybeUndefined<TestStruct>,
}

#[test]
fn test_query_building_null() {
    use cynic::QueryBuilder;

    insta::assert_snapshot!(serde_json::to_string(
        &TestQuery::build(TestArgs { a_str: None.into() }).variables
    )
    .unwrap());
}

#[test]
fn test_query_building_some() {
    use cynic::QueryBuilder;

    insta::assert_snapshot!(serde_json::to_string(
        &TestQuery::build(TestArgs {
            a_str: Some("1").into(),
        })
        .variables
    )
    .unwrap());
}

#[test]
fn test_query_building_undefined() {
    use cynic::QueryBuilder;

    insta::assert_snapshot!(serde_json::to_string(
        &TestQuery::build(TestArgs {
            a_str: Default::default(),
        })
        .variables
    )
    .unwrap());
}
