use cynic::QueryFragment;
use serde::Serialize;
use serde_json::json;

mod query_dsl {
    cynic::query_dsl!("tests/test-schema.graphql");
}

mod recursive_lists {
    use super::*;
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
        #[cynic(recurse = "2")]
        posts: Option<Vec<Post>>,
    }

    #[test]
    fn test_all_posts_query_output() {
        use cynic::{FragmentContext, Operation};

        let query = Operation::query(AllPostsQuery::fragment(FragmentContext::empty()));

        insta::assert_display_snapshot!(query.query);
    }

    #[test]
    fn test_decoding_with_matching_depth() {
        use cynic::{FragmentContext, GraphQlResponse, Operation};

        let data = GraphQlResponse {
            data: Some(json!({ "allPosts": posts(posts(posts(json!(null)))) })),
            errors: None,
        };

        let query = Operation::query(AllPostsQuery::fragment(FragmentContext::empty()));

        insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
    }

    #[test]
    fn test_decoding_with_extra_depth() {
        use cynic::{FragmentContext, GraphQlResponse, Operation};

        let data = GraphQlResponse {
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
}

mod optional_recursive_types {
    use super::*;

    #[derive(QueryFragment, Serialize)]
    #[cynic(
        graphql_type = "Query",
        schema_path = "tests/test-schema.graphql",
        query_module = "query_dsl"
    )]
    struct FriendsQuery {
        all_authors: Vec<Author>,
    }

    #[derive(QueryFragment, Serialize)]
    #[cynic(
        graphql_type = "Author",
        schema_path = "tests/test-schema.graphql",
        query_module = "query_dsl"
    )]
    struct Author {
        #[cynic(recurse = "2")]
        friends: Option<Vec<Author>>,

        #[cynic(recurse = "2")]
        referrer: Box<Option<Author>>,
    }

    #[test]
    fn test_friends_query_output() {
        use cynic::{FragmentContext, Operation};

        let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

        insta::assert_display_snapshot!(query.query);
    }

    macro_rules! null {
        () => {
            json!(null)
        };
    }

    #[test]
    fn test_friends_decoding_with_matching_depth() {
        use cynic::{FragmentContext, GraphQlResponse, Operation};

        let data = GraphQlResponse {
            data: Some(json!({
                "allAuthors":
                    authors(
                        authors(authors(null!(), null!()), author(null!(), null!())),
                        author(authors(null!(), null!()), author(null!(), null!())),
                    )
            })),
            errors: None,
        };

        let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

        insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
    }

    #[test]
    fn test_friends_decoding_with_less_depth() {
        // This is only a valid test for optional fields.
        use cynic::{FragmentContext, GraphQlResponse, Operation};

        let data = GraphQlResponse {
            data: Some(json!({
                "allAuthors": authors(authors(null!(), null!()), author(null!(), null!()))
            })),
            errors: None,
        };

        let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

        insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
    }

    #[test]
    fn test_friends_decoding_with_extra_depth() {
        use cynic::{FragmentContext, GraphQlResponse, Operation};

        let data = GraphQlResponse {
            data: Some(json!({
                "allAuthors":
                    authors(authors(authors(authors(null!(), null!()), null!()), null!()), null!())
            })),
            errors: None,
        };

        let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

        insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
    }

    fn authors(
        inner_friends: serde_json::Value,
        inner_referrer: serde_json::Value,
    ) -> serde_json::Value {
        let a = author(inner_friends, inner_referrer);
        json!([a])
    }

    fn author(
        inner_friends: serde_json::Value,
        inner_referrer: serde_json::Value,
    ) -> serde_json::Value {
        json!({ "friends": inner_friends, "referrer": inner_referrer })
    }
}

mod required_recursive_types {
    use super::*;

    #[derive(QueryFragment, Serialize)]
    #[cynic(
        graphql_type = "Query",
        schema_path = "tests/test-schema.graphql",
        query_module = "query_dsl"
    )]
    struct FriendsQuery {
        all_authors: Vec<Author>,
    }

    #[derive(QueryFragment, Serialize)]
    #[cynic(
        graphql_type = "Author",
        schema_path = "tests/test-schema.graphql",
        query_module = "query_dsl"
    )]
    struct Author {
        #[cynic(recurse = "2")]
        me: Option<Box<Author>>,
    }

    #[test]
    fn test_friends_query_output() {
        use cynic::{FragmentContext, Operation};

        let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

        insta::assert_display_snapshot!(query.query);
    }

    macro_rules! null {
        () => {
            json!(null)
        };
    }

    #[test]
    fn test_friends_decoding_with_matching_depth() {
        use cynic::{FragmentContext, GraphQlResponse, Operation};

        let data = GraphQlResponse {
            data: Some(json!({ "allAuthors": authors(author(author(null!()))) })),
            errors: None,
        };

        let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

        insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
    }

    #[test]
    fn test_friends_decoding_with_less_depth() {
        // This is only a valid test for optional fields.
        use cynic::{FragmentContext, GraphQlResponse, Operation};

        let data = GraphQlResponse {
            data: Some(json!({ "allAuthors": authors(author(null!())) })),
            errors: None,
        };

        let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

        insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
    }

    #[test]
    fn test_friends_decoding_with_extra_depth() {
        use cynic::{FragmentContext, GraphQlResponse, Operation};

        let data = GraphQlResponse {
            data: Some(json!({
                "allAuthors": authors(author(author(author(null!()))))
            })),
            errors: None,
        };

        let query = Operation::query(FriendsQuery::fragment(FragmentContext::empty()));

        insta::assert_yaml_snapshot!(query.decode_response(data).unwrap().data)
    }

    fn authors(me: serde_json::Value) -> serde_json::Value {
        let a = author(me);
        json!([a])
    }

    fn author(me: serde_json::Value) -> serde_json::Value {
        json!({ "me": me })
    }
}
