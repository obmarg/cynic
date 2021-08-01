//! An example of querying the starwars API using the query autogenerating API.

mod gql_schema {
    cynic::use_schema!("../schemas/books.graphql");

    pub fn query<T>(query: cynic::Operation<T>) -> cynic::GraphQlResponse<T>
    where
        T: 'static,
    {
        use cynic::http::ReqwestBlockingExt;

        reqwest::blocking::Client::new()
            .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
            .run_graphql(query)
            .unwrap()
    }

    pub fn subscribe<T>(query: cynic::StreamingOperation<T>) -> cynic::GraphQlResponse<T>
    where
        T: 'static,
    {
        todo!()
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

fn main() {
    // let result = books::query();
    // println!("{:?}", result);

    // let result = create::query();
    // println!("{:?}", result);

    let result = interval::query(42);
    println!("{:?}", result);
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
