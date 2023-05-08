fn main() {}

#[cynic::schema_for_derives(
    file = r#"./../../../../schemas/starwars.schema.graphql"#,
    module = "schema"
)]
mod queries {
    use super::schema;

    #[derive(cynic::QueryVariables)]
    pub struct Variables {
        id: i32,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Root", variables = "Variables")]
    pub struct AllFilms {
        #[arguments(id: $id)]
        pub film: Option<Film>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Film {
        pub id: cynic::Id,
        pub title: Option<String>,
    }
}

mod schema {
    cynic::use_schema!(r#"./../../../../schemas/starwars.schema.graphql"#);
}
