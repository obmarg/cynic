#![allow(unused_imports)]

fn main() {}

#[cynic::schema_for_derives(
    file = r#"./../../../schemas/starwars.schema.graphql"#,
    module = "schema"
)]
mod queries {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Film {
        #[cynic(rename = "episodeID")]
        episode_id: Option<u64>,
    }
}

mod schema {
    cynic::use_schema!(r#"./../../../schemas/starwars.schema.graphql"#);
}
