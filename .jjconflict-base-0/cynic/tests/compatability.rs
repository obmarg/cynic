//! Some tests of cynics implementation of backwards/forwards compatibility

use cynic::{InputObject, QueryFragment};

#[test]
fn query_fields_may_be_unnecessarily_option() {
    // Fields in a query can be wrapped in `Option` when the underlying field
    // is required

    #[allow(dead_code)]
    #[derive(QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct BlogPost {
        author: Option<Author>,
        comments: Option<Vec<Option<Comment>>>,
    }

    #[allow(dead_code)]
    #[derive(QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Author {
        name: Option<String>,
    }

    #[allow(dead_code)]
    #[derive(QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Comment {
        author: Option<Author>,
    }
}

#[test]
fn input_fields_may_be_required_when_nullable() {
    // Input fields can be required when the underlying type is nullable.

    #[allow(dead_code)]
    #[derive(InputObject)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct BlogPostInput {
        author: String,
        content: String,
    }
}

#[test]
fn input_fields_may_be_singular_when_should_be_list() {
    // Input fields can be singular when the underlying type is a list.

    #[allow(dead_code)]
    #[derive(InputObject)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct BlogPostInput {
        content: String,
    }

    #[derive(InputObject)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct SeriesOfBlogs {
        authors: Vec<String>,
        blogs: BlogPostInput,
    }
}

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}
