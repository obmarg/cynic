use std::path::PathBuf;

use similar_asserts::assert_eq;

#[test]
fn directive_args() {
    roundtrip_test("tests/executables/directive_args.graphql");
    display_test("tests/executables/directive_args.graphql");
}

#[test]
fn fragment() {
    roundtrip_test("tests/executables/fragment.graphql");
    display_test("tests/executables/fragment.graphql");
}

#[test]
fn fragment_spread() {
    roundtrip_test("tests/executables/fragment_spread.graphql");
    display_test("tests/executables/fragment_spread.graphql");
}

#[test]
fn inline_fragment() {
    roundtrip_test("tests/executables/inline_fragment.graphql");
    display_test("tests/executables/inline_fragment.graphql");
}

#[test]
fn inline_fragment_dir() {
    roundtrip_test("tests/executables/inline_fragment_dir.graphql");
    display_test("tests/executables/inline_fragment_dir.graphql");
}

#[test]
fn kitchen_sink() {
    double_roundtrip_test("tests/executables/kitchen-sink.graphql");
    display_test("tests/executables/kitchen-sink.graphql");
}

#[test]
fn kitchen_sink_canonical() {
    roundtrip_test("tests/executables/kitchen-sink_canonical.graphql");
    display_test("tests/executables/kitchen-sink_canonical.graphql");
}

#[test]
fn minimal() {
    roundtrip_test("tests/executables/minimal.graphql");
    display_test("tests/executables/minimal.graphql");
}

#[test]
fn minimal_mutation() {
    roundtrip_test("tests/executables/minimal_mutation.graphql");
    display_test("tests/executables/minimal_mutation.graphql");
}

#[test]
fn minimal_query() {
    double_roundtrip_test("tests/executables/minimal_query.graphql");
    display_test("tests/executables/minimal_query.graphql");
}

#[test]
fn multiline_string() {
    double_roundtrip_test("tests/executables/multiline_string.graphql");
    display_test("tests/executables/multiline_string.graphql");
}

#[test]
fn mutation_directive() {
    roundtrip_test("tests/executables/mutation_directive.graphql");
    display_test("tests/executables/mutation_directive.graphql");
}

#[test]
fn named_query() {
    roundtrip_test("tests/executables/named_query.graphql");
    display_test("tests/executables/named_query.graphql");
}

#[test]
fn nested_selection() {
    roundtrip_test("tests/executables/nested_selection.graphql");
    display_test("tests/executables/nested_selection.graphql");
}

#[test]
fn query_aliases() {
    roundtrip_test("tests/executables/query_aliases.graphql");
    display_test("tests/executables/query_aliases.graphql");
}

#[test]
fn query_arguments() {
    insta::assert_snapshot!(
        "query_arguments_double_roundtrip",
        double_roundtrip_test("tests/executables/query_arguments.graphql")
    );
    display_test("tests/executables/query_arguments.graphql");
}

#[test]
fn query_directive() {
    roundtrip_test("tests/executables/query_directive.graphql");
    display_test("tests/executables/query_directive.graphql");
}

#[test]
fn query_list_argument() {
    roundtrip_test("tests/executables/query_list_argument.graphql");
    display_test("tests/executables/query_list_argument.graphql");
}

#[test]
fn query_object_argument() {
    roundtrip_test("tests/executables/query_object_argument.graphql");
    display_test("tests/executables/query_object_argument.graphql");
}

#[test]
fn query_var_default_float() {
    roundtrip_test("tests/executables/query_var_default_float.graphql");
    display_test("tests/executables/query_var_default_float.graphql");
}

#[test]
fn query_var_default_list() {
    roundtrip_test("tests/executables/query_var_default_list.graphql");
}

#[test]
fn query_var_default_object() {
    roundtrip_test("tests/executables/query_var_default_object.graphql");
    display_test("tests/executables/query_var_default_object.graphql");
}

#[test]
fn query_var_default_string() {
    roundtrip_test("tests/executables/query_var_default_string.graphql");
    display_test("tests/executables/query_var_default_string.graphql");
}

#[test]
fn query_var_defaults() {
    roundtrip_test("tests/executables/query_var_defaults.graphql");
    display_test("tests/executables/query_var_defaults.graphql");
}

#[test]
fn query_vars() {
    roundtrip_test("tests/executables/query_vars.graphql");
    display_test("tests/executables/query_vars.graphql");
}

#[test]
fn string_literal() {
    roundtrip_test("tests/executables/string_literal.graphql");
    display_test("tests/executables/string_literal.graphql");
}

#[test]
fn string_escaping() {
    insta::assert_snapshot!(double_roundtrip_test(
        "tests/executables/string_escaping.graphql"
    ));
    display_test("tests/executables/string_escaping.graphql")
}

#[test]
fn subscription_directive() {
    roundtrip_test("tests/executables/subscription_directive.graphql");
    display_test("tests/executables/subscription_directive.graphql");
}

#[test]
fn variable_directive() {
    roundtrip_test("tests/executables/variable_directive.graphql");
    display_test("tests/executables/variable_directive.graphql");
}

fn roundtrip_test(filename: &str) {
    let data = std::fs::read_to_string(filename).unwrap();
    let ast = cynic_parser::parse_executable_document(&data)
        .map_err(|error| error.to_report(&data))
        .unwrap();

    let output = ast.to_string_pretty();

    assert_eq!(data, output);
}

fn display_test(filename: &str) {
    let output = display_roundtrip_test(filename);

    let filename = filename.parse::<PathBuf>().unwrap();
    let snapshot_name = filename.file_stem().unwrap().to_str().unwrap();

    insta::assert_snapshot!(snapshot_name, output);
}

fn display_roundtrip_test(filename: &str) -> String {
    let data = std::fs::read_to_string(filename).unwrap();
    let ast = cynic_parser::parse_executable_document(&data)
        .map_err(|error| error.to_report(&data))
        .unwrap();

    let expected = ast.to_string_pretty();
    let display_output = ast.to_string();

    let ast2 = cynic_parser::parse_executable_document(&display_output)
        .map_err(|error| error.to_report(&data))
        .unwrap();

    let output = ast2.to_string_pretty();

    assert_eq!(output, expected);

    display_output
}

fn double_roundtrip_test(filename: &str) -> String {
    // In some cases the file on disk is not the same as what we output
    // but we still want to make sure we can parse it.
    //
    // For those cases we do a double roundtrip instead of just one
    let data = std::fs::read_to_string(filename).unwrap();
    let ast = cynic_parser::parse_executable_document(&data)
        .map_err(|error| error.to_report(&data))
        .unwrap();

    let round_one_output = ast.to_string_pretty();

    let ast = cynic_parser::parse_executable_document(&round_one_output)
        .map_err(|error| error.to_report(&data))
        .unwrap();

    let round_two_output = ast.to_string_pretty();

    assert_eq!(round_one_output, round_two_output);

    round_two_output
}
