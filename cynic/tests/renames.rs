use cynic::{InlineFragments2, QueryFragment2};
use serde::Serialize;
use serde_json::json;

#[derive(QueryFragment2, Serialize)]
#[cynic(
    graphql_type = "Query",
    schema_path = "tests/test-schema.graphql",
    query_module = "schema"
)]
struct AllPostsQuery {
    all_posts: Vec<Post>,
    all_data: Vec<PostOrAuthor>,
}

#[derive(QueryFragment2, Serialize)]
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

#[derive(QueryFragment2, Serialize)]
#[cynic(
    graphql_type = "Author",
    schema_path = "tests/test-schema.graphql",
    query_module = "schema"
)]
struct Author {
    name: Option<String>,
}

#[derive(InlineFragments2, Serialize)]
#[cynic(schema_path = "tests/test-schema.graphql")]
enum PostOrAuthor {
    #[cynic(rename = "BlogPost")]
    Post(Post),
    Author(Author),
}

#[derive(QueryFragment2, Serialize)]
#[cynic(schema_path = "tests/test-schema.graphql")]
struct EmptyType {
    #[cynic(rename = "_")]
    underscore: Option<bool>,
}

mod schema {
    cynic::use_schema_2!("tests/test-schema.graphql");
}

#[test]
fn test_all_posts_query_output() {
    use cynic::Operation2;

    let query = Operation2::<AllPostsQuery>::query();

    insta::assert_display_snapshot!(query.query);
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
