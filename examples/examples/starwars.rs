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

#[derive(Clone, cynic::FragmentArguments)]
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
    #[cynic_arguments(id = args.id.clone())]
    film: Option<Film>,
}

fn main() {
    let query = build_query();

    let response = reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.com/.netlify/functions/index")
        .json(&query)
        .send()
        .unwrap();

    println!("{:?}", response);

    let result = query.decode_response(response.json().unwrap()).unwrap();

    println!("{:?}", result);
}

fn build_query() -> cynic::Query<'static, FilmDirectorQuery> {
    use cynic::QueryFragment;
    cynic::Query::new(FilmDirectorQuery::fragment(FilmArguments {
        id: Some("ZmlsbXM6MQ==".into()),
    }))
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
}
