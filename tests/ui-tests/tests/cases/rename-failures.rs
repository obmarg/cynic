fn main() {}

mod schema {
    cynic::use_schema!("../../../schemas/starwars.schema.graphql");
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "../../../schemas/starwars.schema.graphql")]
struct Film {
    #[cynic(rename = "episode")]
    episode_id: Option<i32>,
}
