fn main() {}

#[cynic::query_module(
    schema_path = r#"./../../../schemas/starwars.schema.graphql"#,
    query_module = "schema"
)]
mod queries {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Root")]
    pub struct AllFilms {
        pub all_films: Option<FilmsConnection>,
        #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
        pub fiim: Option<Film>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct FilmsConnection {
        pub films: Option<Vec<Option<Film>>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Film {
        pub id: cynic::Id,
        pub title: Option<String>,
    }
}

mod schema {
    cynic::use_schema!(r#"./../../../schemas/starwars.schema.graphql"#);
}
