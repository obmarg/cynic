//! An example of inlining variables into a GraphQL operation prior to sending
//!
//! This example uses the starwars API but the use case is primarily to support
//! shopifies [bulkOperationRunQuery][1] which requires a document with no variables.
//!
//! [1]: https://shopify.dev/docs/api/admin-graphql/2024-07/mutations/bulkoperationrunquery

use cynic::OperationBuilder;

// Pull in the Star Wars schema we registered in build.rs
#[cynic::schema("starwars")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
struct Film {
    title: Option<String>,
    director: Option<String>,
}

#[derive(cynic::QueryVariables, cynic::QueryVariableLiterals)]
struct FilmVariables {
    id: Option<cynic::Id>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Root", variables = "FilmVariables")]
struct FilmDirectorQuery {
    #[arguments(id: $id)]
    film: Option<Film>,
}

fn main() {
    match run_query().data {
        Some(FilmDirectorQuery { film: Some(film) }) => {
            println!("{:?} was directed by {:?}", film.title, film.director)
        }
        _ => {
            println!("No film found");
        }
    }
}

fn run_query() -> cynic::GraphQlResponse<FilmDirectorQuery> {
    use cynic::http::ReqwestBlockingExt;

    let query = build_query();

    reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
        .unwrap()
}

fn build_query() -> cynic::Operation<FilmDirectorQuery, ()> {
    OperationBuilder::query()
        .with_variables(FilmVariables {
            id: Some("ZmlsbXM6MQ==".into()),
        })
        .build_with_variables_inlined()
        .unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn snapshot_test_query() {
        // Running a snapshot test of the query building functionality as that gives us
        // a place to copy and paste the actual GQL we're using for running elsewhere,
        // and also helps ensure we don't change queries by mistake

        let query = build_query();

        insta::assert_snapshot!(query.query);
    }

    #[test]
    fn test_running_query() {
        let result = run_query();
        if result.errors.is_some() {
            assert_eq!(result.errors.unwrap().len(), 0);
        }
        insta::assert_debug_snapshot!(result.data);
    }
}
