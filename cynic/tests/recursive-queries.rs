use cynic::QueryFragment;
use serde::Serialize;
use serde_json::json;

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

mod recursive_lists {
    use super::*;
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
        comments: Vec<Comment>,
    }

    #[derive(QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql", query_module = "schema")]
    struct Comment {
        author: Author,
    }

    #[derive(QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql", query_module = "schema")]
    struct Author {
        #[cynic(recurse = "2")]
        posts: Option<Vec<Post>>,
    }

    #[test]
    fn test_all_posts_query_output() {
        use cynic::QueryBuilder;

        let operation = AllPostsQuery::build(());

        insta::assert_display_snapshot!(operation.query);
    }

    #[test]
    fn test_decoding_with_matching_depth() {
        let data = json!({ "allPosts": posts(posts(posts(json!(null)))) });

        insta::assert_yaml_snapshot!(serde_json::from_value::<AllPostsQuery>(data).unwrap());
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
        query_module = "schema"
    )]
    struct FriendsQuery {
        all_authors: Vec<Author>,
    }

    #[derive(QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql", query_module = "schema")]
    struct Author {
        #[cynic(recurse = "2")]
        friends: Option<Vec<Author>>,

        #[cynic(recurse = "2")]
        referrer: Box<Option<Author>>,
    }

    #[test]
    fn test_friends_query_output() {
        use cynic::QueryBuilder;

        let operation = FriendsQuery::build(());

        insta::assert_display_snapshot!(operation.query);
    }

    macro_rules! null {
        () => {
            json!(null)
        };
    }

    #[test]
    fn test_friends_decoding_with_matching_depth() {
        let data = json!({
            "allAuthors":
                authors(
                    authors(authors(null!(), null!()), author(null!(), null!())),
                    author(authors(null!(), null!()), author(null!(), null!())),
                )
        });
        insta::assert_yaml_snapshot!(serde_json::from_value::<FriendsQuery>(data).unwrap());
    }

    #[test]
    fn test_friends_decoding_with_less_depth() {
        // This is only a valid test for optional fields.
        let data =
            json!({ "allAuthors": authors(authors(null!(), null!()), author(null!(), null!())) });

        insta::assert_yaml_snapshot!(serde_json::from_value::<FriendsQuery>(data).unwrap());
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
        query_module = "schema"
    )]
    struct FriendsQuery {
        all_authors: Vec<Author>,
    }

    #[derive(QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql", query_module = "schema")]
    struct Author {
        #[cynic(recurse = "2")]
        me: Option<Box<Author>>,
    }

    #[test]
    fn test_friends_query_output() {
        use cynic::QueryBuilder;

        let operation = FriendsQuery::build(());

        insta::assert_display_snapshot!(operation.query);
    }

    macro_rules! null {
        () => {
            json!(null)
        };
    }

    #[test]
    fn test_friends_decoding_with_matching_depth() {
        let data = json!({ "allAuthors": authors(author(author(null!()))) });

        insta::assert_yaml_snapshot!(serde_json::from_value::<FriendsQuery>(data).unwrap());
    }

    #[test]
    fn test_friends_decoding_with_less_depth() {
        // This is only a valid test for optional fields.

        let data = json!({ "allAuthors": authors(author(null!())) });

        insta::assert_yaml_snapshot!(serde_json::from_value::<FriendsQuery>(data).unwrap());
    }

    fn authors(me: serde_json::Value) -> serde_json::Value {
        let a = author(me);
        json!([a])
    }

    fn author(me: serde_json::Value) -> serde_json::Value {
        json!({ "me": me })
    }
}
