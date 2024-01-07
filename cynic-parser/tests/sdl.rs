use similar_asserts::assert_eq;

use cynic_parser::Ast;

#[test]
fn test_enum() {
    roundtrip_test("tests/sdl/enum.graphql");
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
