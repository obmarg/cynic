use insta::assert_snapshot;

use cynic_querygen::{document_to_fragment_structs, QueryGenOptions};

macro_rules! test_query_file {
    ($name:ident, $filename:literal) => {
        #[test]
        fn $name() {
            let schema = include_str!("../../schemas/starwars.schema.graphql");
            let query = include_str!(concat!("queries/starwars/", $filename));

            assert_snapshot!(document_to_fragment_structs(
                query,
                schema,
                &QueryGenOptions::default()
            )
            .expect("QueryGen Failed"))
        }
    };
}

test_query_file!(sanity_test_starwars_query, "sanity.graphql");
test_query_file!(test_nested_arguments, "nested-arguments.graphql");
test_query_file!(bare_selection_sets, "bare-selection-set.graphql");
test_query_file!(multiple_queries, "multiple-queries.graphql");
test_query_file!(fragment_spreads, "fragment-spreads.graphql");
test_query_file!(aliases, "aliases.graphql");
test_query_file!(float, "float.graphql");
