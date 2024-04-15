use similar_asserts::assert_eq;

#[test]
fn test_enum() {
    roundtrip_test("tests/sdl/enum.graphql");
}

#[test]
fn directive() {
    roundtrip_test("tests/sdl/directive.graphql");
}

#[test]
fn directive_wrapping() {
    roundtrip_test("tests/sdl/directive_wrapping.graphql");
}

#[test]
fn directive_descriptions() {
    roundtrip_test("tests/sdl/directive_descriptions.graphql");
}

#[test]
fn empty_union() {
    roundtrip_test("tests/sdl/empty_union.graphql");
}

#[test]
fn r#enum() {
    roundtrip_test("tests/sdl/enum.graphql");
}

#[test]
fn extend_enum() {
    roundtrip_test("tests/sdl/extend_enum.graphql");
}

#[test]
fn extend_input() {
    double_roundtrip_test("tests/sdl/extend_input.graphql");
}

#[test]
fn extend_input_canonical() {
    roundtrip_test("tests/sdl/extend_input_canonical.graphql");
}

#[test]
fn extend_interface() {
    roundtrip_test("tests/sdl/extend_interface.graphql");
}

#[test]
fn extend_object() {
    roundtrip_test("tests/sdl/extend_object.graphql");
}

#[test]
fn extend_scalar() {
    roundtrip_test("tests/sdl/extend_scalar.graphql");
}

#[test]
fn field_description() {
    roundtrip_test("tests/sdl/field_description.graphql");
}

#[test]
fn implements() {
    roundtrip_test("tests/sdl/implements.graphql");
}

#[test]
fn implements_amp() {
    roundtrip_test("tests/sdl/implements_amp.graphql");
}

#[test]
fn implements_amp_canonical() {
    roundtrip_test("tests/sdl/implements_amp_canonical.graphql");
}

#[test]
fn input_type() {
    roundtrip_test("tests/sdl/input_type.graphql");
}

#[test]
fn interface() {
    roundtrip_test("tests/sdl/interface.graphql");
}

#[test]
fn kitchen_sink() {
    double_roundtrip_test("tests/sdl/kitchen-sink.graphql");
}

#[test]
fn kitchen_sink_canonical() {
    roundtrip_test("tests/sdl/kitchen-sink_canonical.graphql");
}

#[test]
fn long_argument_wrapping() {
    roundtrip_test("tests/sdl/long_argument_wrapping.graphql");
}

#[test]
fn minimal() {
    roundtrip_test("tests/sdl/minimal.graphql");
}

#[test]
fn minimal_type() {
    roundtrip_test("tests/sdl/minimal_type.graphql");
}

#[test]
fn object_with_arguemnts() {
    roundtrip_test("tests/sdl/object_with_arguments.graphql");
}

#[test]
fn repeatable_directives() {
    roundtrip_test("tests/sdl/repeatable_directives.graphql");
}

#[test]
fn scalar_type() {
    roundtrip_test("tests/sdl/scalar_type.graphql");
}

#[test]
fn simple_object() {
    roundtrip_test("tests/sdl/simple_object.graphql");
}

#[test]
fn union() {
    roundtrip_test("tests/sdl/union.graphql");
}

#[test]
fn string_escaping() {
    insta::assert_snapshot!(double_roundtrip_test("tests/sdl/string_escaping.graphql"))
}

#[test]
fn union_extension() {
    roundtrip_test("tests/sdl/union_extension.graphql");
}

#[test]
fn schema_definition_directives_only() {
    roundtrip_test("tests/sdl/schema_definition_directives_only.graphql");
}

fn roundtrip_test(filename: &str) {
    let data = std::fs::read_to_string(filename).unwrap();
    let ast = cynic_parser::parse_type_system_document(&data).unwrap();

    let output = ast.to_sdl();

    assert_eq!(data, output);
}

fn double_roundtrip_test(filename: &str) -> String {
    // In some cases the file on disk is not the same as what we output
    // but we still want to make sure we can parse it.
    //
    // For those cases we do a double roundtrip instead of just one
    let data = std::fs::read_to_string(filename).unwrap();
    let ast = cynic_parser::parse_type_system_document(&data).unwrap();

    let round_one_output = ast.to_sdl();

    let ast = cynic_parser::parse_type_system_document(&round_one_output).unwrap();

    let round_two_output = ast.to_sdl();

    assert_eq!(round_one_output, round_two_output);

    round_two_output
}
