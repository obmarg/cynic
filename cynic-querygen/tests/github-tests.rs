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

test_query!(add_comment_mutation, "add-comment-mutation.graphql");
test_query!(field_on_interface, "field-on-interface.graphql");
test_query!(fragment_on_interface, "fragment-on-interface.graphql");
test_query!(inline_fragment_on_union, "inline-fragment-on-union.graphql");
test_query!(
    inline_fragment_with_arguments,
    "inline-fragment-with-arguments.graphql"
);
test_query!(
    inline_fragment_with_renames,
    "inline-fragment-with-renames.graphql"
);
test_query!(input_object_arguments, "input-object-arguments.graphql");
test_query!(input_object_literals, "input-object-literals.graphql");
test_query!(literal_enums, "literal-enums.graphql");
test_query!(queries_with_typename, "queries-with-typename.graphql");

test_query!(issue_786, "issue-786.graphql");
