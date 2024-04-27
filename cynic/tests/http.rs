use cynic::QueryBuilder;

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

#[derive(cynic::QueryVariables, Debug)]
pub struct FieldWithStringVariables<'a> {
    pub input: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "FieldWithStringVariables")]
#[cynic(schema_path = "tests/test-schema.graphql")]
pub struct FieldWithString {
    #[arguments(input: $input)]
    pub field_with_string: i32,
}

#[derive(serde::Deserialize)]
struct Extensions {
    code: u16,
}

#[cfg_attr(feature = "http-reqwest", tokio::test)]
async fn test_reqwest_extensions() {
    use cynic::http::ReqwestExt;

    let mut graphql = mockito::Server::new_async().await;

    let response_with_extension = graphql
        .mock("GET", "/graphql")
        .with_body(
            r#"{
            "errors": [
                {
                    "message": "Unauthorized",
                    "locations": null,
                    "path": [
                        "helloWorld"
                    ],
                    "extensions": {
                        "code": 401
                    }
                }
            ]
        }"#,
        )
        .create();

    let client = reqwest::Client::new();
    let output = client
        .get(format!("http://{}/graphql", graphql.host_with_port()))
        .run_graphql(FieldWithString::build(FieldWithStringVariables {
            input: "InputGoesHere",
        }))
        .retain_extensions::<Extensions>()
        .await;
    assert!(output.is_ok());

    let err = output.unwrap();
    assert!(err.data.is_none());

    let errors = err.errors.unwrap();

    let error = &errors[0];
    assert!(matches!(error.extensions, Some(Extensions { code: 401 })));

    response_with_extension.assert();
}

#[cfg_attr(feature = "http-reqwest", tokio::test)]
async fn test_reqwest_ignored() {
    use cynic::http::ReqwestExt;

    let mut graphql = mockito::Server::new_async().await;

    let response_with_extension = graphql
        .mock("GET", "/graphql")
        .with_body(
            r#"{
            "errors": [
                {
                    "message": "Unauthorized",
                    "locations": null,
                    "path": [
                        "helloWorld"
                    ],
                    "extensions": {
                        "code": 401
                    }
                }
            ]
        }"#,
        )
        .create();

    let client = reqwest::Client::new();
    let output = client
        .get(format!("http://{}/graphql", graphql.host_with_port()))
        .run_graphql(FieldWithString::build(FieldWithStringVariables {
            input: "InputGoesHere",
        }))
        .await;
    assert!(output.is_ok());

    let err = output.unwrap();
    assert!(err.data.is_none());

    let errors = err.errors.unwrap();

    let error = &errors[0];
    assert!(matches!(error.extensions, Some(serde::de::IgnoredAny)));

    response_with_extension.assert();
}
