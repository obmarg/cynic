//! Some tests of cynics implementation of backwards/forwards compatability

use cynic::{InputObject, QueryFragment};

#[test]
fn query_fields_may_be_unnecesarily_option() {
    // Fields in a query can be wrapped in `Option` when the underlying field
    // is required

    #[derive(QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct BlogPost {
        author: Option<Author>,
        comments: Option<Vec<Option<Comment>>>,
    }

    #[derive(QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Author {
        name: Option<String>,
    }

    #[derive(QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Comment {
        author: Option<Author>,
    }
}

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}
