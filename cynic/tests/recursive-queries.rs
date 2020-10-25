use cynic::QueryFragment;
use serde_json::json;

mod query_dsl {
    cynic::query_dsl!("tests/test-schema.graphql");
}

#[derive(QueryFragment)]
#[cynic(
    graphql_type = "Query",
    schema_path = "tests/test-schema.graphql",
    query_module = "query_dsl"
)]
struct Query {
    all_posts: Vec<Post>,
}

#[derive(QueryFragment)]
#[cynic(
    graphql_type = "BlogPost",
    schema_path = "tests/test-schema.graphql",
    query_module = "query_dsl"
)]
struct Post {
    comments: Vec<Comment>,
}

#[derive(QueryFragment)]
#[cynic(
    graphql_type = "Comment",
    schema_path = "tests/test-schema.graphql",
    query_module = "query_dsl"
)]
struct Comment {
    author: Author,
}

#[derive(QueryFragment)]
#[cynic(
    graphql_type = "Author",
    schema_path = "tests/test-schema.graphql",
    query_module = "query_dsl"
)]
struct Author {
    #[cynic(recurse = "2")]
    posts: Option<Vec<Post>>,
}

#[test]
fn test_query_output() {
    use cynic::{FragmentContext, Operation};

    let query = Operation::query(Query::fragment(FragmentContext::empty()));

    insta::assert_display_snapshot!(query.query);
}

#[test]
fn test_decoding_of_results() {
    use cynic::{FragmentContext, Operation};

    let query = Operation::query(Query::fragment(FragmentContext::empty()));

    todo!()
}
