//! An example of using subscriptions with async-tungstenite

mod query_dsl {
    cynic::query_dsl!("../schemas/books.graphql");
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/books.graphql",
    query_module = "query_dsl",
    graphql_type = "Book"
)]
struct Book {
    id: String,
    name: String,
    author: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/books.graphql",
    query_module = "query_dsl",
    graphql_type = "BookChanged"
)]
struct BookChanged {
    id: cynic::Id,
    book: Option<Book>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/books.graphql",
    query_module = "query_dsl",
    graphql_type = "SubscriptionRoot"
)]
struct BooksChangedSubscription {
    books: BookChanged,
}

#[async_std::main]
async fn main() {
    use async_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue};
    use cynic::protocol::transport_ws::AsyncWebsocketClient;
    use futures::StreamExt;

    let mut request = "ws://localhost:8000".into_client_request().unwrap();
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_str("graphql-transport-ws").unwrap(),
    );

    let (connection, _) = async_tungstenite::async_std::connect_async(request)
        .await
        .unwrap();

    println!("Connected");

    let mut client = AsyncWebsocketClient::new(connection, async_executors::AsyncStd)
        .await
        .unwrap();

    let mut stream = client.streaming_operation(build_query()).await;
    println!("Running subscription apparently?");
    while let Some(item) = stream.next().await {
        println!("{:?}", item);
    }
}

fn build_query() -> cynic::StreamingOperation<'static, BooksChangedSubscription> {
    use cynic::SubscriptionBuilder;

    BooksChangedSubscription::build(())
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
        let result = run_query();
        if result.errors.is_some() {
            assert_eq!(result.errors.unwrap().len(), 0);
        }
        insta::assert_debug_snapshot!(result.data);
    }
}
