### Sending HTTP Request Manually

The `cynic::http` module provides integrations for some HTTP clients, but
sometimes you might want to make a request manually: either because you're
using a client that `cynic` doesn't support, or the provided integrations just
aren't sufficient in some way.

It's simple to make an HTTP query manually with `cynic`:

- `cynic::Operation` implements `serde::Serialize` to build the body of a
  GraphQL request. This can be used with whatever JSON encoding functionality
  your HTTP client provides.
- The `cynic::QueryFragment` derive generates a `serde::Deserialize` impl that
  you can use to deserialize a `cynic::GraphQlResponse<YourQueryFragment>`

For instance, to make a request with the `reqwest::blocking` client:

```rust
use cynic::{QueryBuilder, GraphQlResponse};

let operation = AllFilmsQuery::build(());

let response = reqwest::blocking::Client::new()
    .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
    .json(&operation)
    .send()
    .unwrap();

let all_films_result = response.json::<GraphQlResponse<AllFilmsQuery>>.unwrap();
```

Now you can do whatever you want with the result.
