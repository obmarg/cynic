mod query_dsl {
    cynic::query_dsl!("examples/examples/starwars.schema.graphql");
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "examples/examples/starwars.schema.graphql",
    query_module = "query_dsl",
    graphql_type = "Film"
)]
struct Film {
    title: Option<String>,
    director: Option<String>,
}

#[derive(cynic::FragmentArguments)]
struct FilmArguments {
    id: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "examples/examples/starwars.schema.graphql",
    query_module = "query_dsl",
    graphql_type = "Root",
    argument_struct = "FilmArguments"
)]
struct FilmDirectorQuery {
    #[cynic_arguments(id = args.id.clone())]
    film: Option<Film>,
}

fn main() {
    use cynic::QueryFragment;

    let query = cynic::Query::new(FilmDirectorQuery::fragment(FilmArguments {
        id: Some("ZmlsbXM6MQ==".to_string()),
    }));

    let response = reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.com/.netlify/functions/index")
        .json(&query.body().unwrap())
        .send()
        .unwrap();

    println!("{:?}", response);

    let result = query.decode_response(response.json().unwrap()).unwrap();

    println!("{:?}", result);
}
