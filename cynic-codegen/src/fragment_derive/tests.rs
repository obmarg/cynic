use std::{io::Write, process::Stdio};

use insta::assert_snapshot;
use rstest::rstest;
use syn::parse_quote;

use super::fragment_derive;

#[rstest(input => [
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
    ),
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
    ),
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
    ),
    parse_quote!(
        #[cynic(
            schema_path = "../cynic/tests/test-schema.graphql",
            schema_module = "schema",
            graphql_type = "Query",
            argument_struct = "AnArgumentStruct"
        )]
        struct MyQuery {
            #[arguments(filters: $filters)]
            filteredPosts: Vec<BlogPostOutput>,
        }
    ),
    parse_quote!(
        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(
            schema_path = "../schemas/starwars.schema.graphql",
            query_module = "schema"
        )]
        struct Film {
            #[cynic(spread)]
            details: FilmDetails,
        }
    ),
    parse_quote!(
        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(
            schema_path = "../schemas/starwars.schema.graphql",
            query_module = "schema"
        )]
        struct Film {
            #[cynic(flatten)]
            producers: String,
        }
    ),
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
])]
fn snapshot_fragment_derive(input: syn::DeriveInput) {
    let tokens = fragment_derive(&input).unwrap();

    assert_snapshot!(format_code(format!("{}", tokens)));
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
