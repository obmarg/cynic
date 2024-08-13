use cynic::QueryFragment;
use serde::Serialize;
use serde_json::json;

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

mod recursive_lists {
    use super::*;
    #[derive(QueryFragment, Serialize)]
    #[cynic(graphql_type = "Query", schema_path = "tests/test-schema.graphql")]
    struct AllPostsQuery {
        all_posts: Vec<Post>,
    }

    #[derive(QueryFragment, Serialize)]
    #[cynic(graphql_type = "BlogPost", schema_path = "tests/test-schema.graphql")]
    struct Post {
        comments: Vec<Comment>,
    }

    #[derive(QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Comment {
        author: Author,
    }

    #[derive(QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
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
    #[cynic(graphql_type = "Query", schema_path = "tests/test-schema.graphql")]
    struct FriendsQuery {
        all_authors: Vec<Author>,
    }

    #[derive(QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
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

    macro_rules! empty_object {
        () => {
            json!(null)
        };
    }

    #[test]
    fn test_friends_decoding_with_matching_depth() {
        let data = json!({
            "allAuthors":
                authors(
                    authors(authors(null!(), null!()), author(empty_object!(), empty_object!())),
                    author(authors(null!(), null!()), author(empty_object!(), empty_object!())),
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
    #[cynic(graphql_type = "Query", schema_path = "tests/test-schema.graphql")]
    struct FriendsQuery {
        all_authors: Vec<Author>,
    }

    #[derive(QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    struct Author {
        #[cynic(recurse = "2")]
        me: Option<Box<Author>>,
    }

    #[test]
    fn test_friends_query_output() {
        use cynic::QueryBuilder;

        let operation = FriendsQuery::build(());

        insta::assert_display_snapshot!(operation.query, @r###"
        query FriendsQuery {
          allAuthors {
            me {
              me
            }
          }
        }

        "###);
    }

    #[test]
    fn test_friends_decoding_with_matching_depth() {
        let data = json!({ "allAuthors": [{"me": {"me": {}}}]});

        insta::assert_yaml_snapshot!(serde_json::from_value::<FriendsQuery>(data).unwrap());
    }

    #[test]
    fn test_friends_decoding_with_less_depth() {
        let data = json!({ "allAuthors": [{"me": null}]});

        insta::assert_yaml_snapshot!(serde_json::from_value::<FriendsQuery>(data).unwrap());
    }
}

mod recursing_through_inline_fragments {
    use cynic_proc_macros::InlineFragments;

    #[test]
    fn test_inline_fragment_preserves_recurse_behaviour() {
        use super::*;

        #[derive(QueryFragment, Serialize)]
        #[cynic(graphql_type = "Query", schema_path = "tests/test-schema.graphql")]
        struct AllDataQuery {
            all_data: Vec<PostOrAuthor>,
        }

        #[derive(InlineFragments, Serialize)]
        #[cynic(schema_path = "tests/test-schema.graphql")]
        enum PostOrAuthor {
            Author(Author),
            #[cynic(fallback)]
            Other,
        }

        #[derive(QueryFragment, Serialize)]
        #[cynic(schema_path = "tests/test-schema.graphql")]
        struct Author {
            #[cynic(recurse = "2")]
            silly_me: Option<Box<PostOrAuthor>>,
        }

        use cynic::QueryBuilder;

        let operation = AllDataQuery::build(());

        insta::assert_display_snapshot!(operation.query, @r###"
        query AllDataQuery {
          allData {
            __typename
            ... on Author {
              sillyMe {
                __typename
                ... on Author {
                  sillyMe {
                    __typename
                  }
                }
              }
            }
          }
        }

        "###);
    }
}

mod recursing_without_recursse {
    use cynic_proc_macros::InlineFragments;

    #[test]
    #[should_panic(
        expected = "Maximum query depth exceeded.  Have you forgotten to mark a query as recursive?"
    )]
    fn test_recursion_without_recurse_panics_correctly() {
        // This example _should_ hit a panic I've added rather than just overflowing
        // the stack.  This test makes sure it does.
        use super::*;

        #[derive(QueryFragment, Serialize)]
        #[cynic(graphql_type = "Query", schema_path = "tests/test-schema.graphql")]
        struct AllDataQuery {
            all_data: Vec<PostOrAuthor>,
        }

        #[derive(InlineFragments, Serialize)]
        #[cynic(schema_path = "tests/test-schema.graphql")]
        enum PostOrAuthor {
            Author(Author),
            Post(Post),
            #[cynic(fallback)]
            Other,
        }

        #[derive(QueryFragment, Serialize)]
        #[cynic(graphql_type = "BlogPost", schema_path = "tests/test-schema.graphql")]
        struct Post {
            __typename: String,
        }

        #[derive(QueryFragment, Serialize)]
        #[cynic(schema_path = "tests/test-schema.graphql")]
        struct Author {
            // #[cynic(recurse = "2")]
            #[cynic(flatten)]
            friends: Vec<Author>,
        }

        use cynic::QueryBuilder;

        AllDataQuery::build(());
    }
}
