use cynic_parser::{SchemaCoordinate, parse_schema_coordinate};

#[test]
fn parsing_schemacoordinates() {
    for (input, expected) in &[
        ("Foo", SchemaCoordinate::ty("Foo")),
        ("Foo.bar", SchemaCoordinate::member("Foo", "bar")),
        (
            "Foo.bar(blah:)",
            SchemaCoordinate::argument("Foo", "bar", "blah"),
        ),
        ("@foo", SchemaCoordinate::directive("foo")),
        (
            "@foo(blah:)",
            SchemaCoordinate::directive_argument("foo", "blah"),
        ),
    ] {
        assert_eq!(&parse_schema_coordinate(input).unwrap(), expected);
    }
}

#[test]
fn schema_coordinates_roundtrip() {
    for input in ["Foo", "Foo.bar", "Foo.bar(blah:)", "@foo", "@foo(blah:)"] {
        assert_eq!(&parse_schema_coordinate(input).unwrap().to_string(), input);
    }
}

#[test]
fn parsing_empty_schema_coordinate_fails() {
    insta::assert_debug_snapshot!(parse_schema_coordinate("").unwrap_err(), @"EmptySchemaCoordinate");
}

#[test]
fn parsing_invalid_schema_coordinate_fails() {
    insta::assert_debug_snapshot!(parse_schema_coordinate("Foo::Bar").unwrap_err(), @r#"
    UnrecognizedToken {
        token: (
            3,
            "colon (':')",
            4,
        ),
        expected: [
            "\".\"",
        ],
    }
    "#);
}
