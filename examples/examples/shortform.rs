//! An example of querying the starwars API using the query autogenerating API.

mod gql_schema {
    use std::error::Error;

    use cynic::{http::ReqwestExt, StreamingOperation};
    use graphql_ws_client::{graphql::Cynic, SubscriptionStream};

    cynic::use_schema!("../schemas/books.graphql");

    #[allow(dead_code)]
    pub async fn query<'a, T>(
        query: cynic::Operation<'a, T>,
    ) -> Result<cynic::GraphQlResponse<T>, impl Error + 'static>
    where
        T: 'static,
    {
        reqwest::Client::new()
            .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
            .run_graphql(query)
            .await
    }

    pub async fn subscribe<'a, T>(
        _query: cynic::StreamingOperation<'a, T>,
    ) -> Result<SubscriptionStream<Cynic, StreamingOperation<'_, T>>, graphql_ws_client::Error>
    where
        T: 'static,
    {
        Result::<_, _>::Err(graphql_ws_client::Error::Unknown("".into()))
    }
}

// cynic::gql! {
//     query film_directory_query($id: ID, $filmid: ID) {
//         film(id: $id, filmID: $filmid) {
//             title, #asdasdasd
//             director,
//         }
//     }
// }

// cynic::gql! {
//     query all_films {
//         allFilms {
//             films {
//                 id
//                 title
//             }
//         }
//     }
// }

// cynic::gql! {
//     query books($id: ID) {
//         books {
//             id,
//             name,
//             author,
//         }
//     }
// }

// cynic::gql! {
//     mutation create {
//         createBook(name: "hehexd", author: "tyler1")
//     }
// }

cynic::gql! {
    subscription interval($n: Int!) {
        interval(n: $n)
    }
}

#[tokio::main]
async fn main() {
    // let result = books::query();
    // println!("{:?}", result);

    // let result = create::query();
    // println!("{:?}", result);

    let mut result = interval::subscribe(42).await.unwrap();
    use futures_util::StreamExt;
    println!("{:?}", result.next().await);
}

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
