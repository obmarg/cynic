use insta::assert_snapshot;

use cynic_querygen::{document_to_fragment_structs, QueryGenOptions};

macro_rules! test_query {
    ($name:ident, $filename:literal) => {
        #[test]
        fn $name() {
            let schema = include_str!("schemas/github.graphql");
            let query = include_str!(concat!("queries/github/", $filename));

            assert_snapshot!(document_to_fragment_structs(
                query,
                schema,
                &QueryGenOptions::default()
            )
            .expect("QueryGen Failed"))
        }
    };
}

test_query!(literal_enums, "literal-enums.graphql");
