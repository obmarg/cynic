mod schema {
    cynic::use_schema!("src/bin/simple.graphql");
}

#[derive(cynic::QueryVariables)]
struct TestArgs<'a> {
    a_str: Option<&'a str>,
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(schema_path = "src/bin/simple.graphql", variables = "TestArgs")]
struct TestStruct {
    #[arguments(x: 1, y: $a_str)]
    field_one: String,
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

#[test]
fn test_query_building() {
    use cynic::QueryBuilder;

    let operation = TestQuery::build(TestArgs { a_str: Some("1") });

    insta::assert_snapshot!(operation.query);
}
