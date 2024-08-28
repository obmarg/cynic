#![allow(dead_code)]

mod schema {
    cynic::use_schema!("../schemas/starwars.schema.graphql");
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Film",
    schema_path = "../schemas/starwars.schema.graphql"
)]
struct FilmDetails {
    title: Option<String>,
    director: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Film",
    schema_path = "../schemas/starwars.schema.graphql"
)]
struct FilmMoreDetails {
    release_date: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "../schemas/starwars.schema.graphql")]
struct Film {
    id: cynic::Id,
    #[cynic(spread)]
    details: FilmDetails,
    #[cynic(spread)]
    more_details: FilmMoreDetails,
}

#[derive(cynic::QueryVariables)]
struct FilmArguments {
    id: Option<cynic::Id>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Root",
    variables = "FilmArguments",
    schema_path = "../schemas/starwars.schema.graphql"
)]
struct FilmDirectorQuery {
    #[arguments(id: $id)]
    film: Option<Film>,
}

fn build_query() -> cynic::Operation<FilmDirectorQuery, FilmArguments> {
    use cynic::QueryBuilder;

    FilmDirectorQuery::build(FilmArguments {
        id: Some("ZmlsbXM6MQ==".into()),
    })
}

#[test]
fn snapshot_test_query() {
    // Running a snapshot test of the query building functionality as that gives us
    // a place to copy and paste the actual GQL we're using for running elsewhere,
    // and also helps ensure we don't change queries by mistake

    let query = build_query();

    insta::assert_snapshot!(query.query);
}
