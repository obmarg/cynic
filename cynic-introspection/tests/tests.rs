use cynic::{GraphQlResponse, http::ReqwestExt};
use cynic_introspection::{SchemaError, SpecificationVersion, query::IntrospectionQuery};
use graphql_mocks::mocks;
use serde_json::json;

#[test]
fn test_2018_introspection_query() {
    insta::assert_snapshot!(build_2018_query().query);
}

fn build_2018_query() -> cynic::Operation<IntrospectionQuery, ()> {
    use cynic::QueryBuilder;

    IntrospectionQuery::build(())
}

#[tokio::test]
async fn test_running_2018_query() {
    let mock_server = mocks::swapi::serve().await;

    let query = build_2018_query();

    let result = reqwest::Client::new()
        .post(mock_server.url())
        .run_graphql(query)
        .await
        .unwrap();

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        panic!("Errors executing query: {errors:?}");
    }
    insta::assert_debug_snapshot!(result.data);
}

#[tokio::test]
async fn test_2018_schema_conversion() {
    let mock_server = mocks::swapi::serve().await;

    let query = build_2018_query();

    let result = reqwest::Client::new()
        .post(mock_server.url())
        .run_graphql(query)
        .await
        .unwrap();

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        panic!("Errors executing query: {errors:?}");
    }
    insta::assert_debug_snapshot!(result.data.unwrap().into_schema().unwrap());
}

fn build_2021_query() -> cynic::Operation<IntrospectionQuery, ()> {
    IntrospectionQuery::with_capabilities(SpecificationVersion::October2021.capabilities())
}

#[test]
fn test_2021_introspection_query() {
    insta::assert_snapshot!(build_2021_query().query);
}

#[tokio::test]
async fn test_running_2021_query() {
    let mock_server = mocks::spacex::serve().await;

    let query = build_2021_query();

    let result = reqwest::Client::new()
        .post(mock_server.url())
        .run_graphql(query)
        .await
        .unwrap();

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        panic!("Errors executing query: {errors:?}");
    }
    insta::assert_debug_snapshot!(result.data);
}

#[tokio::test]
async fn test_2021_schema_conversion() {
    let mock_server = mocks::spacex::serve().await;

    let query = build_2021_query();

    let result = reqwest::Client::new()
        .post(mock_server.url())
        .run_graphql(query)
        .await
        .unwrap();

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        panic!("Errors executing query: {errors:?}");
    }
    insta::assert_debug_snapshot!(result.data.unwrap().into_schema().unwrap());
}

#[test]
fn test_deserializing_disabled_introspection() {
    let response = serde_json::from_value::<GraphQlResponse<IntrospectionQuery>>(json!(
        {
            "data": {"__schema": null},
            "errors": [{"message": "Introspection disabled"}]
        }
    ))
    .unwrap();

    assert_eq!(response.errors.unwrap().len(), 1);
    assert_eq!(
        response.data.unwrap().into_schema(),
        Err(SchemaError::IntrospectionQueryFailed)
    );
}
