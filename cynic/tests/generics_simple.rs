mod schema {
    cynic::use_schema!("../schemas/simple.graphql");
}

#[derive(cynic::QueryVariables)]
struct TestArgs<'a> {
    a_str: Option<&'a str>,
}

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
    schema_path = "../schemas/simple.graphql",
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

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(
    schema_path = "../schemas/simple.graphql",
    graphql_type = "Query",
    variables = "TestArgs"
)]
struct GenericInResponseWithoutBounds<T> {
    test_struct: Option<T>,
}

#[test]
fn test_generic_in_response_without_bounds() {
    use cynic::QueryBuilder;

    let operation =
        GenericInResponseWithoutBounds::<TestStruct>::build(TestArgs { a_str: Some("1") });

    insta::assert_snapshot!(operation.query, @r###"
    query GenericInResponseWithoutBounds($aStr: String) {
      testStruct {
        fieldOne(x: 1, y: $aStr)
      }
    }

    "###);
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(schema_path = "../schemas/simple.graphql", graphql_type = "TestStruct")]
struct TestStructWithoutArgs {
    field_one: String,
}

#[derive(cynic::QueryFragment, PartialEq, Debug)]
#[cynic(schema_path = "../schemas/simple.graphql", graphql_type = "Query")]
struct GenericInResponseWithoutBoundsOrArgs<T> {
    test_struct: Option<T>,
}

#[test]
fn test_generic_in_response_without_bounds_or_args() {
    use cynic::QueryBuilder;

    let operation = GenericInResponseWithoutBoundsOrArgs::<TestStructWithoutArgs>::build(());

    insta::assert_snapshot!(operation.query, @r###"
    query GenericInResponseWithoutBoundsOrArgs {
      testStruct {
        fieldOne
      }
    }

    "###);
}
