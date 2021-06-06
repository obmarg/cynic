use cynic::QueryFragment;
use serde::Serialize;
use serde_json::json;

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

#[derive(QueryFragment, Serialize)]
#[cynic(
    graphql_type = "Query",
    schema_path = "tests/test-schema.graphql",
    query_module = "schema"
)]
struct AllPostsQuery {
    all_posts: Vec<Post>,
}

#[derive(QueryFragment, Serialize)]
#[cynic(
    graphql_type = "BlogPost",
    schema_path = "tests/test-schema.graphql",
    query_module = "schema"
)]
struct Post {
    // TODO: UI tests of failure on renames
    #[cynic(rename = "hasMetadata")]
    metadata_present: Option<bool>,
    metadata: Option<EmptyType>,
}

#[derive(QueryFragment, Serialize)]
#[cynic(schema_path = "tests/test-schema.graphql")]
struct EmptyType {
    #[cynic(rename = "_")]
    underscore: Option<bool>,
}

#[test]
fn test_all_posts_query_output() {
    use cynic::{FragmentContext, Operation};

    let query = Operation::query(AllPostsQuery::fragment(FragmentContext::empty()));

    insta::assert_display_snapshot!(query.query);
}

#[test]
fn test_decoding() {
    use cynic::{FragmentContext, GraphQlResponse, Operation};

    let data = GraphQlResponse {
        data: Some(json!({ "allPosts": posts() })),
        errors: None,
    };

    let query = Operation::query(AllPostsQuery::fragment(FragmentContext::empty()));

    insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
}

fn posts() -> serde_json::Value {
    json!([
        {
            "hasMetadata": true,
            "metadata": {
                "_": null
            }
        }
    ])
}
