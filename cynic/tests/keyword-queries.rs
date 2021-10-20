//! Testing the derives with some keyword fields

use serde_json::json;

#[test]
fn test_query_output() {
    use cynic::QueryBuilder;

    let query = queries::KeywordQuery::build(());

    insta::assert_display_snapshot!(query.query);
}

#[test]
fn test_query_decoding() {
    use cynic::{GraphQlResponse, QueryBuilder};

    let query = queries::KeywordQuery::build(());

    let result = query
        .decode_response(GraphQlResponse {
            data: Some(
                json!({"_": false, "async": false, "crate": false, "self": false, "super": false}),
            ),
            errors: None,
        })
        .unwrap();

    insta::assert_yaml_snapshot!(result.data);
}

#[cynic::schema_for_derives(file = r#"../schemas/test_cases.graphql"#, module = "schema")]
mod queries {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug, serde::Serialize)]
    #[cynic(graphql_type = "Foo")]
    pub struct KeywordQuery {
        #[cynic(rename = "_")]
        pub whatevs: Option<bool>,
        #[cynic(rename = "async")]
        pub whatevs2: Option<bool>,
        #[cynic(rename = "crate")]
        pub whatevs3: Option<bool>,
        #[cynic(rename = "self")]
        pub self_: Option<bool>,
        #[cynic(rename = "super")]
        pub super_: Option<bool>,
    }
}

mod schema {
    cynic::use_schema!(r#"../schemas/test_cases.graphql"#);
}
