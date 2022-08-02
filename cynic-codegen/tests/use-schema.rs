use std::{io::Write, path::PathBuf, process::Stdio};

use insta::assert_snapshot;
use rstest::rstest;

use cynic_codegen::use_schema::{use_schema, UseSchemaParams};

// TODO: Rename this file after running snapshots

#[rstest(schema_file => [
    "graphql.jobs.graphql",
    "books.graphql",
    "starwars.schema.graphql",
    "test_cases.graphql",
    "../cynic/src/bin/simple.graphql",
])]
fn snapshot_use_schema_two(schema_file: &str) {
    let schema_path = PathBuf::from("../schemas/").join(schema_file);

    let tokens = use_schema(UseSchemaParams {
        schema_filename: schema_path.to_str().unwrap().to_string(),
    })
    .unwrap();

    let snapshot_name = std::path::Path::new(schema_file)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap();

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
