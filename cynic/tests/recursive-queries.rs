use cynic::QueryFragment;
use serde::Serialize;
use serde_json::json;

mod query_dsl {
    cynic::query_dsl!("tests/test-schema.graphql");
}

#[derive(QueryFragment, Serialize)]
#[cynic(
    graphql_type = "Query",
    schema_path = "tests/test-schema.graphql",
    query_module = "query_dsl"
)]
struct AllPostsQuery {
    all_posts: Vec<Post>,
}

#[derive(QueryFragment, Serialize)]
#[cynic(
    graphql_type = "BlogPost",
    schema_path = "tests/test-schema.graphql",
    query_module = "query_dsl"
)]
struct Post {
    comments: Vec<Comment>,
}

#[derive(QueryFragment, Serialize)]
#[cynic(
    graphql_type = "Comment",
    schema_path = "tests/test-schema.graphql",
    query_module = "query_dsl"
)]
struct Comment {
    author: Author,
}

#[derive(QueryFragment, Serialize)]
#[cynic(
    graphql_type = "Author",
    schema_path = "tests/test-schema.graphql",
    query_module = "query_dsl"
)]
struct Author {
    // TODO: ok, so not sure this makes sense - think recursion can only ever happen on optional
    // types.  Otherwise there's no way for the GQL server to actually return None.  So probably need
    // to update the code to take that into account.  That is not a job for today though....
    #[cynic(recurse = "2")]
    posts: Option<Vec<Post>>,
}

// TODO: Also need a test of Boxed recursive types, and possibly optional recursive types?

#[test]
fn test_all_posts_query_output() {
    use cynic::{FragmentContext, Operation};

    let query = Operation::query(AllPostsQuery::fragment(FragmentContext::empty()));

    insta::assert_display_snapshot!(query.query);
}

#[test]
fn test_decoding_with_matching_depth() {
    use cynic::{FragmentContext, GraphQLResponse, Operation};

    let data = GraphQLResponse {
        data: Some(json!({ "allPosts": posts(posts(posts(json!(null)))) })),
        errors: None,
    };

    let query = Operation::query(AllPostsQuery::fragment(FragmentContext::empty()));

    insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
}

#[test]
fn test_decoding_with_extra_depth() {
    use cynic::{FragmentContext, GraphQLResponse, Operation};

    let data = GraphQLResponse {
        data: Some(json!({
            "allPosts": posts(posts(posts(posts(json!(null)))))
        })),
        errors: None,
    };

    let query = Operation::query(AllPostsQuery::fragment(FragmentContext::empty()));

    insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
}

fn posts(inner_posts: serde_json::Value) -> serde_json::Value {
    json!([
        {
            "comments": [{
                "author": {
                    "posts": inner_posts
                }
            }]
        }
    ])
}

#[derive(QueryFragment, Serialize)]
#[cynic(
    graphql_type = "Query",
    schema_path = "tests/test-schema.graphql",
    query_module = "query_dsl"
)]
struct FriendsQuery {
    all_authors: Vec<AuthorWithFriends>,
}

#[derive(QueryFragment, Serialize)]
#[cynic(
    graphql_type = "Author",
    schema_path = "tests/test-schema.graphql",
    query_module = "query_dsl"
)]
struct AuthorWithFriends {
    #[cynic(recurse = "2")]
    friends: Option<Vec<AuthorWithFriends>>,
}

#[test]
fn test_friends_query_output() {
    use cynic::{FragmentContext, Operation};

    let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

    insta::assert_display_snapshot!(query.query);
}

#[test]
fn test_friends_decoding_with_matching_depth() {
    use cynic::{FragmentContext, GraphQLResponse, Operation};

    let data = GraphQLResponse {
        data: Some(json!({
            "allAuthors": friends(friends(friends(json!(null))))
        })),
        errors: None,
    };

    let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

    insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
}

#[test]
fn test_friends_decoding_with_less_depth() {
    // This is only a valid test for optional fields.
    use cynic::{FragmentContext, GraphQLResponse, Operation};

    let data = GraphQLResponse {
        data: Some(json!({ "allAuthors": friends(friends(json!(null))) })),
        errors: None,
    };

    let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

    insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
}

#[test]
fn test_friends_decoding_with_extra_depth() {
    use cynic::{FragmentContext, GraphQLResponse, Operation};

    let data = GraphQLResponse {
        data: Some(json!({
            "allAuthors": friends(friends(friends(friends(json!(null)))))
        })),
        errors: None,
    };

    let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

    insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
}

fn friends(inner_friends: serde_json::Value) -> serde_json::Value {
    json!([{ "friends": inner_friends }])
}
