use std::collections::HashMap;

use insta::assert_snapshot;

use cynic_querygen::{QueryGenOptions, document_to_fragment_structs};

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

    let mut field_overrides = HashMap::new();
    field_overrides.insert("FieldNameClashes.str".into(), "std::collections::HashMap".into());
    let options = QueryGenOptions {
        field_overrides,
        ..Default::default()
    };

    assert_snapshot!(
        document_to_fragment_structs(query, schema, &options)
            .expect("QueryGen Failed")
    )
}
