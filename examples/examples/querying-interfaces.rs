//! An example of querying for interfaces with an InlineFragment

// Pull in the Star Wars schema we registered in build.rs
#[cynic::schema("starwars")]
mod schema {}

#[derive(cynic::InlineFragments, Debug)]
enum Node {
    Film(Film),
    Planet(Planet),

    #[cynic(fallback)]
    Other(OtherNode),
}

#[derive(cynic::QueryFragment, Debug)]
struct Film {
    __typename: String,
    title: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
struct Planet {
    name: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Node")]
struct OtherNode {
    id: cynic::Id,
}

#[derive(cynic::QueryVariables)]
struct Arguments {
    id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Root", variables = "Arguments")]
struct Query {
    #[arguments(id: $id)]
    node: Option<Node>,
}

fn main() {
    match run_query(
        "https://swapi-graphql.netlify.app/.netlify/functions/index",
        "ZmlsbXM6MQ==".into(),
    )
    .data
    {
        Some(Query {
            node: Some(Node::Planet(planet)),
        }) => {
            println!("Found a planet: {:?}", planet.name);
        }
        Some(Query {
            node: Some(Node::Film(film)),
        }) => {
            println!("Found a film: {:?}", film.title);
        }
        Some(Query {
            node: Some(Node::Other(node)),
        }) => {
            println!("Found something else with the ID {:?}", node.id);
        }
        _ => {
            println!("No node found");
        }
    }
}

fn run_query(url: &str, id: cynic::Id) -> cynic::GraphQlResponse<Query> {
    use cynic::http::ReqwestBlockingExt;

    let query = build_query(id);

    reqwest::blocking::Client::new()
        .post(url)
        .run_graphql(query)
        .unwrap()
}

fn build_query(id: cynic::Id) -> cynic::Operation<Query, Arguments> {
    use cynic::QueryBuilder;

    Query::build(Arguments { id })
}

#[cfg(test)]
mod test {
    use tokio::task::spawn_blocking;

    use super::*;

    #[test]
    fn snapshot_test_query() {
        // Running a snapshot test of the query building functionality as that gives us
        // a place to copy and paste the actual GQL we're using for running elsewhere,
        // and also helps ensure we don't change queries by mistake

        let query = build_query("ZmlsbXM6MQ==".into());

        insta::assert_snapshot!(query.query);
    }

    #[tokio::test]
    async fn test_running_query_with_film() {
        let mock_server = graphql_mocks::mocks::swapi::serve().await;

        let result = spawn_blocking(move || {
            run_query(&mock_server.url().to_string(), "ZmlsbXM6MQ==".into())
        })
        .await
        .unwrap();

        if result.errors.is_some() {
            assert_eq!(
                result.errors.as_ref().unwrap().len(),
                0,
                "Server Errored: {:?}",
                result.errors
            );
        }
        insta::assert_debug_snapshot!(result.data);
    }

    #[tokio::test]
    async fn test_running_query_with_planet() {
        let mock_server = graphql_mocks::mocks::swapi::serve().await;

        let result = spawn_blocking(move || {
            run_query(&mock_server.url().to_string(), "cGxhbmV0czo0OQ==".into())
        })
        .await
        .unwrap();

        if result.errors.is_some() {
            assert_eq!(
                result.errors.as_ref().unwrap().len(),
                0,
                "Server Errored: {:?}",
                result.errors
            );
        }
        insta::assert_debug_snapshot!(result.data);
    }

    #[tokio::test]
    async fn test_running_query_with_starship() {
        let mock_server = graphql_mocks::mocks::swapi::serve().await;

        let result = spawn_blocking(move || {
            run_query(&mock_server.url().to_string(), "c3RhcnNoaXBzOjY1".into())
        })
        .await
        .unwrap();

        if result.errors.is_some() {
            assert_eq!(
                result.errors.as_ref().unwrap().len(),
                0,
                "Server Errored: {:?}",
                result.errors
            );
        }
        insta::assert_debug_snapshot!(result.data);
    }
}
