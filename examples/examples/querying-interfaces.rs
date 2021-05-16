//! An example of querying for interfaces with an InlineFragment

mod schema {
    cynic::use_schema!("../schemas/starwars.schema.graphql");
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(
    schema_path = "../schemas/starwars.schema.graphql",
    query_module = "schema"
)]
enum Node {
    Film(Film),
    Planet(Planet),

    #[cynic(fallback)]
    Other(OtherNode),
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/starwars.schema.graphql",
    query_module = "schema"
)]
struct Film {
    title: Option<String>,
    director: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/starwars.schema.graphql",
    query_module = "schema"
)]
struct Planet {
    name: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/starwars.schema.graphql",
    graphql_type = "Node"
)]
struct OtherNode {
    id: cynic::Id,
}

#[derive(cynic::FragmentArguments)]
struct Arguments {
    id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/starwars.schema.graphql",
    graphql_type = "Root",
    argument_struct = "Arguments"
)]
struct FilmDirectorQuery {
    #[arguments(id = args.id.clone())]
    node: Option<Node>,
}

fn main() {
    let result = run_query("ZmlsbXM6MQ==".into());
    println!("{:?}", result);
}

fn run_query(id: cynic::Id) -> cynic::GraphQlResponse<FilmDirectorQuery> {
    use cynic::http::ReqwestBlockingExt;

    let query = build_query(id);

    reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
        .unwrap()
}

fn build_query(id: cynic::Id) -> cynic::Operation<'static, FilmDirectorQuery> {
    use cynic::QueryBuilder;

    FilmDirectorQuery::build(&Arguments { id })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn snapshot_test_query() {
        // Running a snapshot test of the query building functionality as that gives us
        // a place to copy and paste the actual GQL we're using for running elsewhere,
        // and also helps ensure we don't change queries by mistake

        let query = build_query("ZmlsbXM6MQ==".into());

        insta::assert_snapshot!(query.query);
    }

    #[test]
    fn test_running_query_with_film() {
        let result = run_query("ZmlsbXM6MQ==".into());
        if result.errors.is_some() {
            assert_eq!(result.errors.unwrap().len(), 0);
        }
        insta::assert_debug_snapshot!(result.data);
    }

    #[test]
    fn test_running_query_with_planet() {
        let result = run_query("cGxhbmV0czo0OQ==".into());
        if result.errors.is_some() {
            assert_eq!(result.errors.unwrap().len(), 0);
        }
        insta::assert_debug_snapshot!(result.data);
    }

    #[test]
    fn test_running_query_with_starship() {
        let result = run_query("c3RhcnNoaXBzOjY1".into());
        if result.errors.is_some() {
            assert_eq!(result.errors.unwrap().len(), 0);
        }
        insta::assert_debug_snapshot!(result.data);
    }
}
