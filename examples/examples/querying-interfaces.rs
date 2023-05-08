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
    match run_query("ZmlsbXM6MQ==".into()).data {
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

fn run_query(id: cynic::Id) -> cynic::GraphQlResponse<Query> {
    use cynic::http::ReqwestBlockingExt;

    let query = build_query(id);

    reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
        .unwrap()
}

fn build_query(id: cynic::Id) -> cynic::Operation<Query, Arguments> {
    use cynic::QueryBuilder;

    Query::build(Arguments { id })
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
