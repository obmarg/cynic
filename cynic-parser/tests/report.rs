use cynic_parser::Error;

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
       │ Note: expected one of "enum", BlockStringLiteral, RawIdent, StringLiteral, directive, extend, false, implements, input, interface, mutation, null, on, query, repeatable, scalar, schema, subscription, true, ty, union
    ───╯
    "###);
}
