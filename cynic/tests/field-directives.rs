//! Tests of skip & include directives

use serde::Serialize;

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

mod skip_directive {
    use serde::Deserialize;
    use serde_json::json;

    use super::*;

    #[derive(cynic::QueryVariables)]
    struct Vars {
        should_skip: bool,
    }

    #[derive(cynic::QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql", variables = "Vars")]
    struct BlogPost {
        #[directives(skip(if: $should_skip))]
        id: Option<cynic::Id>,

        #[directives(skip(if: true))]
        has_metadata: Option<bool>,

        #[directives(skip(if: false))]
        state: Option<PostState>,
    }

    #[derive(cynic::Enum)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    enum PostState {
        Posted,
        Draft,
    }

    #[derive(cynic::QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql", variables = "Vars")]
    struct Query {
        #[allow(dead_code)]
        #[arguments(filters: { states: DRAFT })]
        filtered_posts: Vec<BlogPost>,
    }

    #[test]
    fn test_query() {
        use cynic::QueryBuilder;

        let query = Query::build(Vars { should_skip: true });

        insta::assert_display_snapshot!(query.query, @r###"
        query Query($shouldSkip: Boolean!) {
          filteredPosts(filters: {states: [DRAFT]}) {
            id @skip(if: $shouldSkip)
            hasMetadata @skip(if: true)
            state @skip(if: false)
          }
        }

        "###);
    }

    #[test]
    fn test_deser() {
        let decoded = Query::deserialize(json!({
            "filteredPosts": [
                {},
                {"id": "1", "hasMetadata": true, "state": "DRAFT"}
            ]
        }))
        .unwrap();
        insta::assert_json_snapshot!(decoded, @r###"
        {
          "filtered_posts": [
            {
              "id": null,
              "has_metadata": null,
              "state": null
            },
            {
              "id": "1",
              "has_metadata": true,
              "state": "DRAFT"
            }
          ]
        }
        "###)
    }
}

mod include_directive {
    use serde::Deserialize;
    use serde_json::json;

    use super::*;

    #[derive(cynic::QueryVariables)]
    struct Vars {
        should_include: bool,
    }

    #[derive(cynic::QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql", variables = "Vars")]
    struct BlogPost {
        #[directives(include(if: $should_include))]
        id: Option<cynic::Id>,

        #[directives(include(if: true))]
        has_metadata: Option<bool>,

        #[directives(include(if: false))]
        state: Option<PostState>,
    }

    #[derive(cynic::Enum)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    enum PostState {
        Posted,
        Draft,
    }

    #[derive(cynic::QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql", variables = "Vars")]
    struct Query {
        #[allow(dead_code)]
        #[arguments(filters: { states: DRAFT })]
        filtered_posts: Vec<BlogPost>,
    }

    #[test]
    fn test_query() {
        use cynic::QueryBuilder;

        let query = Query::build(Vars {
            should_include: true,
        });

        insta::assert_display_snapshot!(query.query, @r###"
        query Query($shouldInclude: Boolean!) {
          filteredPosts(filters: {states: [DRAFT]}) {
            id @include(if: $shouldInclude)
            hasMetadata @include(if: true)
            state @include(if: false)
          }
        }

        "###);
    }

    #[test]
    fn test_deser() {
        let decoded = Query::deserialize(json!({
            "filteredPosts": [
                {},
                {"id": "1", "hasMetadata": true, "state": "DRAFT"}
            ]
        }))
        .unwrap();
        insta::assert_json_snapshot!(decoded, @r###"
        {
          "filtered_posts": [
            {
              "id": null,
              "has_metadata": null,
              "state": null
            },
            {
              "id": "1",
              "has_metadata": true,
              "state": "DRAFT"
            }
          ]
        }
        "###)
    }
}

mod other_directives {
    use super::*;

    #[derive(cynic::QueryVariables)]
    struct Vars {
        an_int: i32,
    }

    #[derive(cynic::QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql", variables = "Vars")]
    struct BlogPost {
        #[directives(foo)]
        id: Option<cynic::Id>,

        #[directives(foo(blah: {optionalInt: $an_int}))]
        has_metadata: Option<bool>,

        #[directives(foo(blah: {optionalInt: 1}))]
        state: Option<PostState>,
    }

    #[derive(cynic::Enum)]
    #[cynic(schema_path = "tests/test-schema.graphql")]
    enum PostState {
        Posted,
        Draft,
    }

    #[derive(cynic::QueryFragment, Serialize)]
    #[cynic(schema_path = "tests/test-schema.graphql", variables = "Vars")]
    struct Query {
        #[allow(dead_code)]
        #[arguments(filters: { states: DRAFT })]
        filtered_posts: Vec<BlogPost>,
    }

    #[test]
    fn test_query() {
        use cynic::QueryBuilder;

        let query = Query::build(Vars { an_int: 120 });

        insta::assert_display_snapshot!(query.query, @r###"
        query Query($anInt: Int!) {
          filteredPosts(filters: {states: [DRAFT]}) {
            id @foo
            hasMetadata @foo(blah: {optionalInt: $anInt})
            state @foo(blah: {optionalInt: 1})
          }
        }

        "###);
    }
}
