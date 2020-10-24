//! An example that shows how to make & decode a GraphQL operation using
//! reqwest without the cynic integration code

mod query_dsl {
    cynic::query_dsl!("examples/starwars.schema.graphql");
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "examples/starwars.schema.graphql",
    query_module = "query_dsl",
    graphql_type = "Film"
)]
struct Film {
    title: Option<String>,
    director: Option<String>,
}

#[derive(cynic::FragmentArguments)]
struct FilmArguments {
    id: Option<cynic::Id>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "examples/starwars.schema.graphql",
    query_module = "query_dsl",
    graphql_type = "Root",
    argument_struct = "FilmArguments"
)]
struct FilmDirectorQuery {
    #[arguments(id = &args.id)]
    film: Option<Film>,
}

fn main() {
    let result = run_query();
    println!("{:?}", result);
}

fn run_query() -> cynic::GraphQLResponse<FilmDirectorQuery> {
    let query = build_query();

    let response = reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.com/.netlify/functions/index")
        .json(&query)
        .send()
        .unwrap();

    println!("{:?}", response);

    query.decode_response(response.json().unwrap()).unwrap()
}

fn build_query() -> cynic::Operation<'static, FilmDirectorQuery> {
    use cynic::{FragmentContext, QueryFragment};
    cynic::Operation::query(FilmDirectorQuery::fragment(FragmentContext::with_args(
        &FilmArguments {
            id: Some("ZmlsbXM6MQ==".into()),
        },
    )))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn snapshot_test_menu_query() {
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
