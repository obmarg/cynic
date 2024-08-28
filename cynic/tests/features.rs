use serde_json::json;

#[derive(cynic::QueryFragment, Debug, PartialEq)]
#[cynic(
    schema_path = "../schemas/starwars.schema.graphql",
    graphql_type = "Root"
)]
struct QueryWithFeatures {
    #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
    #[cynic(rename = "film", alias, feature = "a_new_hope")]
    a_new_hope: Option<Film>,

    #[arguments(id = cynic::Id::new("ZmlsbXM6Mg=="))]
    #[cynic(rename = "film", alias)]
    empire_strikes_back: Option<Film>,
}

#[derive(cynic::QueryFragment, Debug, PartialEq)]
#[cynic(schema_path = "../schemas/starwars.schema.graphql")]
struct Film {
    title: Option<String>,
}

mod schema {
    cynic::use_schema!("../schemas/starwars.schema.graphql");
}

#[test]
fn query_string_with_feature_enabled() {
    let operation = cynic::OperationBuilder::<QueryWithFeatures, ()>::query()
        .with_variables(())
        .with_feature_enabled("a_new_hope")
        .build()
        .expect("operation building to succeed");

    insta::assert_snapshot!(operation.query);
}

#[test]
fn query_string_without_feature_enabled() {
    let operation = cynic::OperationBuilder::<QueryWithFeatures, ()>::query()
        .with_variables(())
        .build()
        .expect("operation building to succeed");

    insta::assert_snapshot!(operation.query);
}

#[test]
fn decoding_succeeds_with_data_missing() {
    let data = serde_json::from_value::<QueryWithFeatures>(
        json!({ "empire_strikes_back": {"title": "Empire Strikes Back"} }),
    )
    .expect("deser to succeed");

    assert_eq!(data.a_new_hope, None);
    assert_eq!(
        data.empire_strikes_back,
        Some(Film {
            title: Some("Empire Strikes Back".into())
        })
    );
}

#[test]
fn test_decoding_with_data_present() {
    let data = serde_json::from_value::<QueryWithFeatures>(json!({
        "empire_strikes_back": {"title": "Empire Strikes Back"},
        "a_new_hope": {"title": "A New Hope"}
    }))
    .expect("deser to succeed");

    assert_eq!(
        data.a_new_hope,
        Some(Film {
            title: Some("A New Hope".into())
        })
    );
    assert_eq!(
        data.empire_strikes_back,
        Some(Film {
            title: Some("Empire Strikes Back".into())
        })
    );
}
