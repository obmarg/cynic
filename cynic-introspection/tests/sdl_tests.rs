use cynic_introspection::{query2018, query2021};

#[test]
fn test_starwars_sdl_conversion() {
    use cynic::http::ReqwestBlockingExt;
    use cynic::QueryBuilder;

    let query = query2018::IntrospectionQuery::build(());

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
    use cynic::QueryBuilder;

    let query = query2021::IntrospectionQuery::build(());

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
