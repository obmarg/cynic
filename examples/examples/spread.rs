//! An example of querying the starwars API using the reqwest-blocking feature

// Pull in the Star Wars schema we registered in build.rs
#[cynic::schema("starwars")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Film")]
struct FilmDetails {
    title: Option<String>,
    director: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Film")]
struct FilmMoreDetails {
    release_date: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
struct Film {
    id: cynic::Id,
    #[cynic(spread)]
    details: FilmDetails,
    #[cynic(spread)]
    more_details: FilmMoreDetails,
    producers: Option<Vec<Option<String>>>,
}

#[derive(cynic::QueryVariables)]
struct FilmArguments {
    id: Option<cynic::Id>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Root", variables = "FilmArguments")]
struct FilmDirectorQuery {
    #[arguments(id: $id)]
    film: Option<Film>,
}

fn main() {
    match run_query().data {
        Some(FilmDirectorQuery { film: Some(film) }) => {
            println!("Id: {:?}", film.id);
            println!(
                "{:?} was directed by {:?}",
                film.details.title, film.details.director
            );
            println!("Released on: {:?}", film.more_details.release_date);
            println!("Producers: {:?}", film.producers);
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

fn build_query() -> cynic::Operation<FilmDirectorQuery, FilmArguments> {
    use cynic::QueryBuilder;

    FilmDirectorQuery::build(FilmArguments {
        id: Some("ZmlsbXM6MQ==".into()),
    })
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
