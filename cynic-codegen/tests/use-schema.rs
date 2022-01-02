use std::{io::Write, path::PathBuf, process::Stdio};

use insta::assert_snapshot;
use rstest::rstest;

use cynic_codegen::use_schema::{use_schema, QueryDslParams};
use cynic_codegen::use_schema2::{use_schema as use_schema_2, UseSchemaParams as UseSchema2Params};

// TODO: Rename this file after running snapshots

#[rstest(schema_file => [
    "graphql.jobs.graphql",
    "books.graphql",
    "starwars.schema.graphql",
    "test_cases.graphql",
])]
fn snapshot_use_schema(schema_file: &str) {
    let schema_path = PathBuf::from("../schemas/").join(schema_file);

    let tokens = use_schema(QueryDslParams {
        schema_filename: schema_path.to_str().unwrap().to_string(),
    })
    .unwrap();

    assert_snapshot!(format_code(format!("{}", tokens)));
}

#[rstest(schema_file_two => [
    "graphql.jobs.graphql",
    "books.graphql",
    "starwars.schema.graphql",
    "test_cases.graphql",
    "../cynic/src/bin/simple.graphql"
])]
fn snapshot_use_schema_two(schema_file_two: &str) {
    let schema_path = PathBuf::from("../schemas/").join(schema_file_two);

    let tokens = use_schema_2(UseSchema2Params {
        schema_filename: schema_path.to_str().unwrap().to_string(),
    })
    .unwrap();

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
