use insta::assert_snapshot;

#[test]
fn test_field_overrides() {
    let schema = include_str!("../../schemas/test_cases.graphql");
    let query = r#"
      query MyQuery($input: OneOfObject!) {
        clashes {
          str
          bool
          i32
          u32
        }
      }
    "#;

    let generator = cynic_querygen::Generator::new(schema)
        .expect("schema parse failed")
        .with_override(
            "FieldNameClashes.str",
            "std::collections::HashMap<String, String>",
        )
        .unwrap();

    assert_snapshot!(generator.generate(query).expect("QueryGen Failed"))
}
