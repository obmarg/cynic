use std::{io::Write, process::Stdio};

use insta::assert_snapshot;
use rstest::rstest;
use syn::parse_quote;

use super::input_object_derive;

mod queries {}

#[rstest(input => [
    parse_quote! {
        #[derive(cynic::InputObject, Debug)]
        #[cynic(schema_path = r#"../schemas/github.graphql"#)]
        pub struct IssueOrder {
            pub direction: OrderDirection,
            pub field: IssueOrderField,
        }
    }
])]
fn snapshot_input_object_derive(input: syn::DeriveInput) {
    let tokens = input_object_derive(&input).unwrap();

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
