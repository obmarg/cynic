use cynic::{InlineFragments, QueryFragment};
use serde::Serialize;
use serde_json::json;

#[derive(QueryFragment, Serialize)]
#[cynic(
    graphql_type = "Query",
    schema_path = "tests/test-schema.graphql",
    query_module = "schema"
)]
struct AllPostsQuery {
    all_posts: Vec<Post>,
    all_data: Vec<PostOrAuthor>,
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
#[cynic(
    graphql_type = "Author",
    schema_path = "tests/test-schema.graphql",
    query_module = "schema"
)]
struct Author {
    name: Option<String>,
}

#[derive(InlineFragments, Serialize)]
#[cynic(schema_path = "tests/test-schema.graphql")]
enum PostOrAuthor {
    Post(Post),
    Author(Author),
    #[cynic(fallback)]
    Other,
}

#[derive(QueryFragment, Serialize)]
#[cynic(schema_path = "tests/test-schema.graphql")]
struct EmptyType {
    #[cynic(rename = "_")]
    underscore: Option<bool>,
}

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

#[test]
fn test_all_posts_query_output() {
    use cynic::QueryBuilder;

    let operation = AllPostsQuery::build(());

    insta::assert_display_snapshot!(operation.query);
}

#[test]
fn test_decoding() {
    use cynic::GraphQlResponse;

    let mut all_data = posts();
    all_data[0]["__typename"] = json!("BlogPost");

    let data = json!({
        "data": { "allPosts": posts(), "allData": all_data },
        "errors": null,
    });

    insta::assert_yaml_snapshot!(
        serde_json::from_value::<GraphQlResponse<AllPostsQuery>>(data)
            .unwrap()
            .data
    );
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
