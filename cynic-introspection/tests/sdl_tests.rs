use cynic_introspection::{IntrospectionQuery, SpecificationVersion};

#[test]
fn test_starwars_sdl_conversion() {
    use cynic::http::ReqwestBlockingExt;

    let query =
        IntrospectionQuery::with_capabilities(SpecificationVersion::June2018.capabilities());

    let result = reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
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
        .run_graphql(query)
        .unwrap();

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        panic!("Errors executing query: {errors:?}");
    }
    insta::assert_snapshot!(result.data.unwrap().into_schema().unwrap().to_sdl());
}
