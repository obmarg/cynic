use std::{io::Write, process::Stdio};

use insta::assert_snapshot;
use rstest::rstest;
use syn::parse_quote;

use super::inline_fragments_derive;

#[rstest(input => [
    // A union type
    parse_quote!(
        #[derive(InlineFragments, Serialize)]
        #[cynic(schema_path = "../cynic/tests/test-schema.graphql")]
        enum PostOrAuthor {
            Post(Post),
            Author(Author),
            #[cynic(fallback)]
            Other
        }
    ),
    // An interface
    parse_quote!(
        #[derive(InlineFragments, Serialize)]
        #[cynic(schema_path = "../cynic/tests/test-schema.graphql")]
        enum Node {
            #[cynic(rename = "BlogPost")]
            Post(Post),
            Author(Author),
            #[cynic(fallback)]
            Other
        }
    ),
    // A union that has rename
    parse_quote!(
        #[derive(InlineFragments, Serialize)]
        #[cynic(schema_path = "../cynic/tests/test-schema.graphql")]
        enum PostOrAuthor {
            #[cynic(rename = "BlogPost")]
            Post(Post),
            Author(Author),
            #[cynic(fallback)]
            Other
        }
    ),
])]
fn snapshot_inline_fragments_derive(input: syn::DeriveInput) {
    let tokens = inline_fragments_derive(&input).unwrap();

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
