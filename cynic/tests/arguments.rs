//! Tests of the generated serialization code for InputObjects

use serde::Serialize;

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

#[test]
fn test_literal_object_inside_list() {
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
        #[arguments(filters: { any: [{states: DRAFT}] })]
        filtered_posts: Vec<BlogPost>,
    }

    let query = Query::build(());

    insta::assert_display_snapshot!(query.query, @r###"
    query Query {
      filteredPosts(filters: {any: [{states: [DRAFT]}]}) {
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
