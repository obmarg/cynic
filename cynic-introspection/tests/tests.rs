use cynic_introspection::query::IntrospectionQuery;

#[test]
fn test_introspection_query() {
    insta::assert_snapshot!(build_query().query);
}

fn build_query() -> cynic::Operation<IntrospectionQuery, ()> {
    use cynic::QueryBuilder;

    IntrospectionQuery::build(())
}

#[test]
fn test_running_query() {
    use cynic::http::ReqwestBlockingExt;

    let query = build_query();

    let result = reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
        .unwrap();

    if result.errors.is_some() {
        assert_eq!(result.errors.unwrap().len(), 0);
    }
    insta::assert_debug_snapshot!(result.data);
}

#[test]
fn test_schema_conversion() {
    use cynic::http::ReqwestBlockingExt;

    let query = build_query();

    let result = reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
        .unwrap();

    if result.errors.is_some() {
        assert_eq!(result.errors.unwrap().len(), 0);
    }
    insta::assert_debug_snapshot!(result.data.unwrap().into_schema().unwrap());
}
