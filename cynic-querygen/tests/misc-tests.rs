use insta::assert_snapshot;

use cynic_querygen::{document_to_fragment_structs, QueryGenOptions};

#[test]
fn mutation_with_scalar_result_and_input() {
    let schema = include_str!("../../schemas/raindancer.graphql");
    let query = include_str!("queries/misc/mutation_with_scalar_result_and_input.graphql");

    assert_snapshot!(
        document_to_fragment_structs(query, schema, &QueryGenOptions::default())
            .expect("QueryGen Failed")
    )
}

#[test]
fn book_subscription_test() {
    let schema = include_str!("../../schemas/books.graphql");
    let query = include_str!("queries/misc/books-subscription.graphql");

    assert_snapshot!(
        document_to_fragment_structs(query, schema, &QueryGenOptions::default())
            .expect("QueryGen Failed")
    )
}

#[test]
fn keyword_renames() {
    let schema = include_str!("../../schemas/test_cases.graphql");
    let query = include_str!("queries/misc/keyword-renames.graphql");

    assert_snapshot!(
        document_to_fragment_structs(query, schema, &QueryGenOptions::default())
            .expect("QueryGen Failed")
    )
}

#[test]
fn test_scalar_casing() {
    let schema = include_str!("../../schemas/test_cases.graphql");
    let query = include_str!("queries/misc/scalar-casing.graphql");

    assert_snapshot!(
        document_to_fragment_structs(query, schema, &QueryGenOptions::default())
            .expect("QueryGen Failed")
    )
}

#[test]
fn test_recursive_inputs() {
    let schema = include_str!("../../schemas/test_cases.graphql");
    let query = r#"
      query MyQuery($input: SelfRecursiveInput!, $input2: RecursiveInputParent!) {
        recursiveInputField(recursive: $input, recursive2: $input2)
      }
    "#;

    assert_snapshot!(
        document_to_fragment_structs(query, schema, &QueryGenOptions::default())
            .expect("QueryGen Failed")
    )
}

#[test]
fn test_string_escaping() {
    let schema = include_str!("../../schemas/test_cases.graphql");
    let query = r#"
      query {
        one: fieldWithStringArg(input: "Hello\n There")
        two: fieldWithStringArg(input: "Hello \" I am a string with quotes")
      }
    "#;

    assert_snapshot!(
        document_to_fragment_structs(query, schema, &QueryGenOptions::default())
            .expect("QueryGen Failed")
    )
}

#[test]
fn test_with_named_schema() {
    let schema = include_str!("../../schemas/github.graphql");
    let query = include_str!("queries/github/query-with-all-derives.graphql");

    assert_snapshot!(document_to_fragment_structs(
        query,
        schema,
        &QueryGenOptions {
            schema_name: Some("my-schema".into()),
            ..QueryGenOptions::default()
        }
    )
    .expect("QueryGen Failed"))
}
