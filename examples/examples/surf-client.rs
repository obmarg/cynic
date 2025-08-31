//! An example of querying the starwars API using surf via the cynic
//! integration.
//!
//! This example requires the `surf` feature to be active.

// Pull in the Star Wars schema we registered in build.rs
#[cynic::schema("starwars")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
struct Film {
    title: Option<String>,
    director: Option<String>,
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
    async_std::task::block_on(async {
        match run_query("https://swapi-graphql.netlify.app/.netlify/functions/index")
            .await
            .data
        {
            Some(FilmDirectorQuery { film: Some(film) }) => {
                println!("{:?} was directed by {:?}", film.title, film.director)
            }
            _ => {
                println!("No film found");
            }
        }
    })
}

async fn run_query(url: &str) -> cynic::GraphQlResponse<FilmDirectorQuery> {
    use cynic::http::SurfExt;

    let operation = build_query();

    surf::post(url).run_graphql(&operation).await.unwrap()
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

    // Having serious problems getting this to work on CI and it's not that
    // important so I'm just going to skip it for now :(
    #[cfg(nope)]
    #[test]
    fn test_running_query() {
        async_std::task::block_on(async {
            let result = run_query().await;
            if result.errors.is_some() {
                assert_eq!(result.errors.unwrap().len(), 0);
            }
            insta::assert_debug_snapshot!(result.data);
        });
    }
}
