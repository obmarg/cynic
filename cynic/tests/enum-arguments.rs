//! Tests of the generated serialization code for InputObjects

use serde::Serialize;

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

#[test]
fn test_enum_argument_literal() {
    use cynic::QueryBuilder;

    #[derive(cynic::QueryFragment, Serialize)]
    #[cynic(graphql_type = "BlogPost", schema_path = "tests/test-schema.graphql")]
    struct Post {
        has_metadata: Option<bool>,
    }

    #[derive(cynic::QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Query {
        #[allow(dead_code)]
        #[arguments(filters: { states: DRAFT })]
        filtered_posts: Vec<BlogPost>,
    }

    let query = Query::build(());

    insta::assert_display_snapshot!(query.query, @r###"
    query {
      filteredPosts(filters: {states: [DRAFT, ], }) {
        hasMetadata
      }
    }

    "###);
}

#[test]
fn test_enum_argument() {
    use cynic::QueryBuilder;

    #[derive(cynic::Enum)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    enum PostState {
        Posted,
        Draft,
    }

    #[derive(cynic::InputObject)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct PostFilters {
        states: Option<Vec<PostState>>,
    }

    #[derive(cynic::QueryFragment)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Query {
        #[allow(dead_code)]
        #[arguments(filters = PostFilters { states: Some(vec![PostState::Posted]) })]
        filtered_posts: Vec<BlogPost>,
    }

    let query = Query::build(());

    insta::assert_display_snapshot!(query.query, @r###"
    query {
      filteredPosts(filters: {states: [POSTED, ], }) {
        hasMetadata
      }
    }

    "###);
}

#[derive(cynic::QueryFragment, Serialize)]
#[cynic(schema_path = "tests/test-schema.graphql")]
struct BlogPost {
    has_metadata: Option<bool>,
}
