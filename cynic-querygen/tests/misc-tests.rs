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
