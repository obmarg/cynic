fn main() {}

#[cynic::query_module(
    schema_path = r#"./../../../schemas/starwars.schema.graphql"#,
    query_module = "query_dsl"
)]
mod queries {
    use super::query_dsl;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Root")]
    pub struct AllFilms {
        pub all_films: Option<FilmsConnection>,
        #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
        pub fiim: Option<Film>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "FilmsConnection")]
    pub struct FilmsConnection {
        pub films: Option<Vec<Option<Film>>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Film")]
    pub struct Film {
        pub id: cynic::Id,
        pub title: Option<String>,
    }
}

mod query_dsl {
    cynic::query_dsl!(r#"./../../../schemas/starwars.schema.graphql"#);
}
