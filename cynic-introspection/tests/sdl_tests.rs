use cynic::http::ReqwestExt;
use cynic_introspection::{IntrospectionQuery, SpecificationVersion};
use graphql_mocks::mocks;

#[tokio::test]
async fn test_starwars_sdl_conversion() {
    let mock_server = mocks::swapi::serve().await;

    let query =
        IntrospectionQuery::with_capabilities(SpecificationVersion::June2018.capabilities());

    let result = reqwest::Client::new()
        .post(mock_server.url())
        .run_graphql(&query)
        .await
        .unwrap();

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        panic!("Errors executing query: {errors:?}");
    }
    insta::assert_snapshot!(result.data.unwrap().into_schema().unwrap().to_sdl());
}

#[test]
fn test_spacex_sdl_conversion() {
    use cynic::http::ReqwestBlockingExt;

    let query =
        IntrospectionQuery::with_capabilities(SpecificationVersion::October2021.capabilities());

    let result = reqwest::blocking::Client::new()
        .post("https://spacex-production.up.railway.app/")
        .run_graphql(&query)
        .unwrap();

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        panic!("Errors executing query: {errors:?}");
    }
    insta::assert_snapshot!(result.data.unwrap().into_schema().unwrap().to_sdl());
}
