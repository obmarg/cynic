#[test]
fn test_report() {
    let document = "type Blah {}";
    let report = cynic_parser::parse_type_system_document(document)
        .map(|_| ())
        .unwrap_err()
        .to_report(document);

    insta::assert_display_snapshot!(report, @r###"
    [03] Error: unexpected closing brace ('}')
       ╭─[<unknown>:1:1]
       │
     1 │ type Blah {}
       │            ┬  
       │            ╰── didn't expect to see this
       │ 
       │ Note: expected one of RawIdent, StringLiteral, BlockStringLiteral, schema, query, mutation, subscription, ty, input, true, false, null, implements, interface, "enum", union, scalar, extend, directive, repeatable, on, fragment
    ───╯
    "###);
}

#[test]
fn test_invalid_directive_location() {
    let document = "directive @Blah on BLAH";

    let report = cynic_parser::parse_type_system_document(document)
        .map(|_| ())
        .unwrap_err()
        .to_report(document);

    insta::assert_display_snapshot!(report, @r###"
    [03] Error: unknown directive location: BLAH. expected one of QUERY, MUTATION, SUBSCRIPTION, FIELD, FRAGMENT_DEFINITION, FRAGMENT_SPREAD, INLINE_FRAGMENT, SCHEMA, SCALAR, OBJECT, FIELD_DEFINITION, ARGUMENT_DEFINITION, INTERFACE, UNION, ENUM, ENUM_VALUE, INPUT_OBJECT, INPUT_FIELD_DEFINITION, VARIABLE_DEFINITION
       ╭─[<unknown>:1:1]
       │
     1 │ directive @Blah on BLAH
       │                    ──┬─  
       │                      ╰─── this is not a valid directive location
    ───╯
    "###);
}
