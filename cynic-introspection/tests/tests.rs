use cynic_introspection::{query2018::IntrospectionQuery, query2021};

#[test]
fn test_2018_introspection_query() {
    insta::assert_snapshot!(build_2018_query().query);
}

fn build_2018_query() -> cynic::Operation<IntrospectionQuery, ()> {
    use cynic::QueryBuilder;

    IntrospectionQuery::build(())
}

#[test]
fn test_running_2018_query() {
    use cynic::http::ReqwestBlockingExt;

    let query = build_2018_query();

    let result = reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
        .unwrap();

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        panic!("Errors executing query: {errors:?}");
    }
    insta::assert_debug_snapshot!(result.data);
}

#[test]
fn test_2018_schema_conversion() {
    use cynic::http::ReqwestBlockingExt;

    let query = build_2018_query();

    let result = reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
        .unwrap();

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        panic!("Errors executing query: {errors:?}");
    }
    insta::assert_debug_snapshot!(result.data.unwrap().into_schema().unwrap());
}

fn build_2021_query() -> cynic::Operation<query2021::IntrospectionQuery, ()> {
    use cynic::QueryBuilder;

    query2021::IntrospectionQuery::build(())
}

#[test]
fn test_2021_introspection_query() {
    insta::assert_snapshot!(build_2021_query().query);
}

#[test]
fn test_running_2021_query() {
    use cynic::http::ReqwestBlockingExt;

    let query = build_2021_query();

    let result = reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
        .unwrap();

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        panic!("Errors executing query: {errors:?}");
    }
    insta::assert_debug_snapshot!(result.data);
}

#[test]
fn test_2021_schema_conversion() {
    use cynic::http::ReqwestBlockingExt;

    let query = build_2021_query();

    let result = reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
        .unwrap();

    if result.errors.is_some() {
        let errors = result.errors.unwrap();
        panic!("Errors executing query: {errors:?}");
    }
    insta::assert_debug_snapshot!(result.data.unwrap().into_schema().unwrap());
}
