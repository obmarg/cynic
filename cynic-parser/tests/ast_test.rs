use cynic_parser::{
    ast::{AstNode, ExecutableDef, NameOwner},
    parse_query_document,
};

#[test]
fn test_query_ast() {
    let document = parse_query_document(
        r#"
    query MyQuery($arg: String!) {
      posts {
        title
        content
      }
    }
    "#,
    )
    .unwrap();

    let executables = document.definitions().collect::<Vec<_>>();
    assert_eq!(executables.len(), 1);

    let op = executables.first().unwrap().operation_def().unwrap();
    assert_eq!(op.name().unwrap().to_string(), "MyQuery");

    // Check it's a query
    op.operation_type().unwrap().query_keyword_token().unwrap();

    let selections = op.selection_set().unwrap().selection().collect::<Vec<_>>();

    assert_eq!(selections.len(), 1);
    let field_selection = selections.first().unwrap().field_selection().unwrap();
    assert_eq!(field_selection.name().unwrap().to_string(), "posts");

    let inner_selections = field_selection
        .selection_set()
        .unwrap()
        .selection()
        .map(|m| m.field_selection().unwrap().name().unwrap().to_string())
        .collect::<Vec<_>>();

    assert_eq!(inner_selections, &["title", "content"])
}
