use serde_json::json;

#[derive(cynic::QueryVariables)]
struct TestArgs<'a> {
    #[cynic(skip_serializing_if = "Option::is_none")]
    a_str: Option<&'a str>,
}

#[test]
fn test_skip_serializing_if_none() {
    assert_eq!(
        serde_json::to_value(TestArgs { a_str: None }).unwrap(),
        json!({})
    );
    assert_eq!(
        serde_json::to_value(TestArgs {
            a_str: Some("hello")
        })
        .unwrap(),
        json!({"aStr": "hello"})
    );
}

mod schema {
    cynic::use_schema!("src/bin/simple.graphql");
}
