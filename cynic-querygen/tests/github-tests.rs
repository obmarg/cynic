use insta::assert_snapshot;

use cynic_querygen::{document_to_fragment_structs, QueryGenOptions};

macro_rules! test_query {
    ($name:ident, $filename:literal) => {
        #[test]
        fn $name() {
            let schema = include_str!("../../schemas/github.graphql");
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
test_query!(input_object_literals, "input-object-literals.graphql");
test_query!(input_object_arguments, "input-object-arguments.graphql");
test_query!(add_comment_mutation, "add-comment-mutation.graphql");
test_query!(inline_fragment_on_union, "inline-fragment-on-union.graphql");
