use insta::assert_snapshot;

use cynic_querygen::{document_to_fragment_structs, QueryGenOptions};

#[test]
fn sanity_test_starwars_query() {
    let schema = include_str!("../../examples/examples/starwars.schema.graphql");
    let query = r#"
        query Query($filmId: ID) {
            film(id: $filmId) {
                title
                director
            }
        }
    "#;

    assert_snapshot!(
        document_to_fragment_structs(query, schema, &QueryGenOptions::default())
            .expect("QueryGen Failed")
    )
}
