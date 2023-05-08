fn main() {}
#[cynic::schema_for_derives(
    file = r#"./../../../../schemas/starwars.schema.graphql"#,
    module = "schema"
)]
mod queries {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Root")]
    pub struct MyQuery {
        pub all_films: Option<FilmsConnection>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct FilmsConnection {
        pub edges: Option<Vec<Option<FilmsEdge>>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct FilmsEdge {
        pub node: Option<Film>,
        pub cursor: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Film {
        pub created: Option<String>,
        pub edited: Option<String>,
        pub opening_crawl: Option<String>,
        pub planet_connection: Option<FilmPlanetsConnection>,
        pub release_date: Option<String>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct FilmPlanetsConnection {
        pub page_info: PageInfo,
        pub total_count: Option<i32>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PageInfo {
        pub end_cursor: Option<String>,
        pub has_previous_page: bool,
    }
}

mod schema {
    cynic::use_schema!("./../../../../schemas/starwars.schema.graphql");
}
