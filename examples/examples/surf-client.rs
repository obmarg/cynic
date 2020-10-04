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

#[derive(cynic::FragmentArguments)]
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
    #[arguments(id = &args.id)]
    film: Option<Film>,
}

fn main() {
    async_std::task::block_on(async {
        let result = run_query().await;
        println!("{:?}", result);
    })
}

async fn run_query() -> cynic::GraphQLResponse<FilmDirectorQuery> {
    use cynic::http::SurfExt;

    let operation = build_query();

    surf::post("http://swapi-graphql.netlify.com/.netlify/functions/index")
        .run_graphql(operation)
        .await
        .unwrap()
}

fn build_query() -> cynic::Operation<'static, FilmDirectorQuery> {
    use cynic::QueryFragment;
    cynic::Operation::query(FilmDirectorQuery::fragment(&FilmArguments {
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

    #[test]
    fn test_running_query() {
        async_std::task::block_on(async {
            let result = run_query().await;
            if result.errors.is_some() {
                assert_eq!(result.errors.unwrap().len(), 0);
            }
            insta::assert_debug_snapshot!(result.data);
        });
    }
}
