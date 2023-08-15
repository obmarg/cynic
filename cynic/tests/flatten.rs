#[derive(cynic::QueryFragment, Debug, PartialEq)]
#[cynic(schema_path = "../schemas/starwars.schema.graphql")]
struct Film {
    /// Flatten some scalars
    #[cynic(rename = "producers", flatten)]
    producers_one: Vec<String>,

    #[cynic(rename = "producers", flatten)]
    producers_two: Option<Vec<String>>,
}

#[derive(cynic::QueryFragment, Debug, PartialEq)]
#[cynic(schema_path = "../schemas/starwars.schema.graphql")]
struct FilmCharactersConnection {
    /// Flatten some scalars
    #[cynic(rename = "edges", flatten)]
    edges_one: Vec<FilmCharactersEdge>,

    #[cynic(rename = "edges", flatten)]
    edges_two: Option<Vec<FilmCharactersEdge>>,
}

#[derive(cynic::QueryFragment, Debug, PartialEq)]
#[cynic(schema_path = "../schemas/starwars.schema.graphql")]
struct FilmCharactersEdge {
    cursor: String,
}

#[derive(cynic::QueryFragment, Debug, PartialEq)]
#[cynic(
    schema_path = "../schemas/test_cases.graphql",
    schema_module = "schema_2"
)]
struct FlattenableEnums {
    /// Flatten some scalars
    #[cynic(rename = "states", flatten)]
    states: Vec<States>,

    #[cynic(rename = "states", flatten)]
    states_two: Option<Vec<States>>,
}

#[derive(cynic::Enum, Debug, PartialEq)]
#[cynic(
    schema_path = "../schemas/test_cases.graphql",
    schema_module = "schema_2"
)]
enum States {
    Closed,
    Deleted,
    Open,
}

mod schema {
    cynic::use_schema!("../schemas/starwars.schema.graphql");
}

mod schema_2 {
    cynic::use_schema!("../schemas/test_cases.graphql");
}

// TODO: Write some decoding tests (either here or next to the flatten_de stuff)
