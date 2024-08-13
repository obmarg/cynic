use std::{io::Write, process::Stdio};

use insta::assert_snapshot;
use rstest::rstest;
use syn::parse_quote;

use super::fragment_derive;

#[rstest]
#[case::argument_and_rename(
    "argument_and_rename",
    parse_quote!(
        #[cynic(
            schema_path = "../cynic/tests/test-schema.graphql",
            schema_module = "schema",
            graphql_type = "Query"
        )]
        struct MyQuery {
            #[arguments(id: "1234")]
            post: Option<BlogPostOutput>,
            #[cynic(rename = "allPosts")]
            posts: Vec<BlogPostOutput>,
        }
    )
)]
#[case::simple_struct(
    "simple_struct",
    parse_quote!(
        #[cynic(
            schema_path = "../cynic/tests/test-schema.graphql",
            schema_module = "schema",
            graphql_type = "BlogPost"
        )]
        struct BlogPostOutput {
            has_metadata: Option<bool>,
            author: AuthorOutput,
        }
    )
)]
#[case::not_sure(
    "not_sure",
    parse_quote!(
        #[cynic(
            schema_path = "../cynic/tests/test-schema.graphql",
            schema_module = "schema",
            graphql_type = "Query"
        )]
        struct MyQuery {
            #[arguments(filters: {states: ["POSTED", "DRAFT"]})]
            filteredPosts: Vec<BlogPostOutput>,
        }
    )
)]
#[case::variable_in_argument(
    "variable_in_argument",
    parse_quote!(
        #[cynic(
            schema_path = "../cynic/tests/test-schema.graphql",
            schema_module = "schema",
            graphql_type = "Query",
            variables = "AnArgumentStruct"
        )]
        struct MyQuery {
            #[arguments(filters: $filters)]
            filteredPosts: Vec<BlogPostOutput>,
        }
    )
)]
#[case::spread_attr(
    "spread_attr",
    parse_quote!(
        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(
            schema_path = "../schemas/starwars.schema.graphql",
            schema_module = "schema"
        )]
        struct Film {
            #[cynic(spread)]
            details: FilmDetails,
        }
    )
)]
#[case::spread_attr_multi_field1(
    "spread_attr_multi_field1",
    parse_quote!(
        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(
            schema_path = "../schemas/starwars.schema.graphql",
            schema_module = "schema"
        )]
        struct Film {
            release_date: Option<String>,
            #[cynic(spread)]
            details: FilmDetails,
        }
    )
)]
#[case::spread_attr_multi_field2(
    "spread_attr_multi_field2",
    parse_quote!(
        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(
            schema_path = "../schemas/starwars.schema.graphql",
            schema_module = "schema"
        )]
        struct Film {
            #[cynic(spread)]
            details: FilmDetails,
            release_date: Option<String>,
        }
    )
)]
#[case::flatten_attr(
    "flatten_attr",
    parse_quote!(
        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(
            schema_path = "../schemas/starwars.schema.graphql",
            schema_module = "schema"
        )]
        struct Film {
            #[cynic(flatten)]
            producers: Vec<String>,
        }
    )
)]
#[case::argument_literals(
    "argument_literals",
    parse_quote!(
        #[cynic(
            schema_path = "../cynic/tests/test-schema.graphql",
            schema_module = "schema",
            graphql_type = "Query"
        )]
        struct MyQuery {
            #[arguments(filters: {states: ["POSTED", "POSTED"]})]
            filteredPosts: Vec<BlogPostOutput>,
        }
    ),
)]
#[case::feature_flagging(
    "feature_flagging",
    parse_quote!(
        #[cynic(
            schema_path = "../cynic/tests/test-schema.graphql",
            schema_module = "schema",
            graphql_type = "Query"
        )]
        struct MyQuery {
            __typename: String,
            #[cynic(feature = "2018")]
            filteredPosts: Vec<BlogPostOutput>,
        }
    ),
)]
fn snapshot_fragment_derive(#[case] snapshot_name: &str, #[case] input: syn::DeriveInput) {
    let tokens = fragment_derive(&input).unwrap();

    assert_snapshot!(snapshot_name, format_code(format!("{}", tokens)));
}

fn format_code(input: String) -> String {
    let mut cmd = std::process::Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to execute rustfmt");

    write!(cmd.stdin.as_mut().unwrap(), "{}", input).unwrap();

    std::str::from_utf8(&cmd.wait_with_output().unwrap().stdout)
        .unwrap()
        .to_owned()
}
