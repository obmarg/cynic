fn main() {}

#[derive(cynic::QueryFragment, Debug, PartialEq)]
#[cynic(schema_path = "./../../../../schemas/starwars.schema.graphql")]
struct Film {
    /// Flatten some scalars wrong
    #[cynic(rename = "producers", flatten)]
    producers_one: Vec<i32>,

    #[cynic(rename = "producers", flatten)]
    producers_two: Option<Vec<f64>>,
}

#[derive(cynic::QueryFragment, Debug, PartialEq)]
#[cynic(schema_path = "./../../../../schemas/starwars.schema.graphql")]
struct FilmCharactersConnection {
    /// Flatten some scalars
    #[cynic(rename = "edges", flatten)]
    edges_one: Vec<String>,
}

mod schema {
    cynic::use_schema!(r#"./../../../../schemas/starwars.schema.graphql"#);
}
