use std::{io::Write, path::PathBuf, process::Stdio};

use insta::assert_snapshot;
use rstest::rstest;
use syn::parse_quote;

use super::fragment_derive;

// TODO: Rename this file after running snapshots

#[rstest(input => [
    parse_quote!(
        #[cynic(
            schema_path = "../cynic/tests/test-schema.graphql",
            schema_module = "schema",
            graphql_type = "Query"
        )]
        struct MyQuery {
            #[arguments(id = "TODO")]
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
    )
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
