//! An example of querying the starwars API using the reqwest-blocking feature

mod gql_schema {
    cynic::use_schema!("../schemas/starwars.schema.graphql");

    pub fn run_query<T>(query: cynic::Operation<T>) -> cynic::GraphQlResponse<T>
    where
        T: 'static,
    {
        use cynic::http::ReqwestBlockingExt;

        reqwest::blocking::Client::new()
            .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
            .run_graphql(query)
            .unwrap()
    }
}

cynic::gql!(
    "
    query film_directory_query($id: ID) {
        films(id: $id) {
            title,
            director,
        }
    }
"
);

fn main() {
    // let result = run_query();
    // println!("{:?}", result);
}

// fn run_query() -> cynic::GraphQlResponse<FilmDirectorQuery> {
//     use cynic::http::ReqwestBlockingExt;

//     let query = build_query();

//     reqwest::blocking::Client::new()
//         .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
//         .run_graphql(query)
//         .unwrap()
// }

// fn build_query() -> cynic::Operation<'static, FilmDirectorQuery> {
//     use cynic::QueryBuilder;

//     FilmDirectorQuery::build(&FilmArguments {
//         id: Some("ZmlsbXM6MQ==".into()),
//     })
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn snapshot_test_query() {
        // Running a snapshot test of the query building functionality as that gives us
        // a place to copy and paste the actual GQL we're using for running elsewhere,
        // and also helps ensure we don't change queries by mistake

        let query = build_query();

        insta::assert_snapshot!(query.query);
    }

    #[test]
    fn test_running_query() {
        let result = run_query();
        if result.errors.is_some() {
            assert_eq!(result.errors.unwrap().len(), 0);
        }
        insta::assert_debug_snapshot!(result.data);
    }
}
