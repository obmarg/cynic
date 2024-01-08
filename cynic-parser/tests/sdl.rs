use similar_asserts::assert_eq;

use cynic_parser::Ast;

#[test]
fn test_enum() {
    roundtrip_test("tests/sdl/enum.graphql");
}

#[test]
fn directive() {
    roundtrip_test("tests/sdl/directive.graphql");
}

#[test]
#[ignore]
fn directive_descriptions() {
    roundtrip_test("tests/sdl/directive_descriptions.graphql");
}

#[test]
#[ignore]
fn directive_descriptions_canonical() {
    roundtrip_test("tests/sdl/directive_descriptions_canonical.graphql");
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
#[ignore]
fn extend_enum() {
    roundtrip_test("tests/sdl/extend_enum.graphql");
}

#[test]
#[ignore]
fn extend_input() {
    roundtrip_test("tests/sdl/extend_input.graphql");
}

#[test]
#[ignore]
fn extend_input_canonical() {
    roundtrip_test("tests/sdl/extend_input_canonical.graphql");
}

#[test]
#[ignore]
fn extend_interface() {
    roundtrip_test("tests/sdl/extend_interface.graphql");
}

#[test]
#[ignore]
fn extend_object() {
    roundtrip_test("tests/sdl/extend_object.graphql");
}

#[test]
#[ignore]
fn extend_scalar() {
    roundtrip_test("tests/sdl/extend_scalar.graphql");
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
#[ignore]
fn kitchen_sink() {
    roundtrip_test("tests/sdl/kitchen-sink.graphql");
}

#[test]
#[ignore]
fn kitchen_sink_canonical() {
    roundtrip_test("tests/sdl/kitchen-sink_canonical.graphql");
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
#[ignore]
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
#[ignore]
fn union_extension() {
    roundtrip_test("tests/sdl/union_extension.graphql");
}

#[allow(dead_code)]
fn parse_test(filename: &str) -> Ast {
    let data = std::fs::read_to_string(filename).unwrap();
    cynic_parser::parse_type_system_document(&data)
}

fn roundtrip_test(filename: &str) {
    let data = std::fs::read_to_string(filename).unwrap();
    let ast = cynic_parser::parse_type_system_document(&data);

    let output = ast.to_sdl();

    assert_eq!(data, output);
}
