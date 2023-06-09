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

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    graphql_type = "Query",
    variables = "TestArgs"
)]
struct GenericInResponse<T>
where
    T: cynic::QueryFragment<SchemaType = schema::TestStruct>,
    TestArgsFields: cynic::queries::VariableMatch<T::VariablesFields>,
{
    test_struct: Option<T>,
}

#[test]
fn test_generic_in_response() {
    use cynic::QueryBuilder;

    let operation = GenericInResponse::<TestStruct>::build(TestArgs { a_str: Some("1") });

    insta::assert_snapshot!(operation.query, @r###"
    query GenericInResponse($aStr: String) {
      testStruct {
        fieldOne(x: 1, y: $aStr)
      }
    }

    "###);
}
