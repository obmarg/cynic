# Quickstart

If you just want to get going with cynic and don't care too much about how it
works, this is the chapter for you.

#### Pre-requisites

There's a few things you'll need before you get started:

1. An existing rust project (though you can just run `cargo new` if you don't
   have one).
2. A GraphQL API that you'd like to query, and a copy of it's schema.
3. A GraphQL query that you'd like to run against the API. If you don't have
   one of these, you should probably use graphiql or graphql playground to get
   started, or you can use the one I provide below.  For this quickstart I'll be
   assuming you're using a query without any arguments.

#### Setting up dependencies.

First things first: you need to add cynic to your dependencies. We'll also need
an HTTP client library.  For the purposes of this quickstart we'll be using
reqwest, but you can use any library you want.  Open up your `Cargo.toml` and
add the following under the `[dependencies]` section:

```toml
cynic = "0.6"
reqwest = { version = "0.10.1", features = ["json", "blocking"] }
```

You may also optionally want to install `insta` - a snapshot testing library
that can be useful for double checking your GraphQL queries are as expected.
Add this under `[dev-dependencies]` so it's available under test but not at
runtime:

```toml
[dev-dependencies]
insta = "0.16"
```

Run a `cargo check` to make sure this builds and you're good to go.

#### Adding your schema to the build.

You'll want to make sure the GraphQL schema for your build is available to your
builds.  For example, you could put it at `src/schema.graphql` - the rest of
this tutorial will assume that's where you put the schema.

#### Building your query structs.

Cynic allows you to build queries from Rust structs - so you'll need to take
the query you're wanting to run, and convert it into one or more rust structs.
This can be quite laborious and error prone for larger queries though so cynic
provides [`querygen`][1] to help you get started.

Go to [https://generator.cynic-rs.dev][1] (please excuse the UI) and enter your
schema (in GraphQL SDL) and your GraphQL query in the left most 2 textareas.
The right most text area should now contain some Rust code. You should copy and
paste this into a file in your repository.  Ensure that the `schema_path`
listed matches the path to your GraphQL schema locally, relative to your
`Cargo.toml` - so with the example it would be `src/schema.graphql`

For example, I've chosen to add the star wars schema and the following query:

```graphql
TODO
```

and been given, the following rust code:

```rust
TODO
```

#### Checking your query (optional)

Since cynic generates queries for you based on Rust structs, it's not always
obvious what the GraphQL queries look like.  Sometimes you might want to run a
query manually via Graphiql, or you might just want to see what effects
changing the rust structs have on the query itself.

I find writing snapshot tests using `insta` useful for this purpose.  Assuming
your query is called `AllFilmsQuery` like mine is, you can add the following to
the same file you put the struct output into:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_films_query_gql_output() {
        use cynic::QueryFragment;
        let query = cynic::Query::new(AllFilmsQuery::fragment(()));
        insta::assert_snapshot!(query.query);
    }
}
```

You can now run this test with `cargo test`.  It should fail the first time, as
you've not yet saved a snapshot.  Run `cargo insta review` (you may need to
`cargo install insta` first) to approve the snapshot, and the test should succeed.

You can use this snapshot test to double check the query whenever you make
changes to the rust code, or just when you need some actual GraphQL to make a
query outside of cynic.

#### Making your query

Now, you're ready to make a query against a server.  Cynic doesn't provide any
HTTP code for you, so you'll need to reach for your HTTP library of choice for
this one.  We'll use reqwest here, but it should be similar for any others.

First, you'll want to build a `Query` similar to how we did it in the snapshot
test above (again, swapping `AllFilmsQuery` for the name of your root query
struct):

```rust
use cynic::QueryFragment;
let query = cynic::Query::new(AllFilmsQuery::fragment());
```

This `Query` struct is serializable using `serde::Serialize`, so you should
pass it in as the HTTP body using your HTTP client and then make a request.
For example, to use reqwest to talk to the StarWars API:

```rust
let response = reqwest::blocking::Client::new()
    .post("https://swapi-graphql.netlify.com/.netlify/functions/index")
    .json(&query)
    .send()
    .unwrap();
```

Now, assuming everything went well, you should have a response containing JSON.
First you need to decode this JSON, then you can pass it to the
`decode_response` function which will handle decoding into your query structs:

```rust
let all_films_result = query.decode_response(response.json().unwrap()).unwrap();
```

Now you can do whatever you want with the results of your query.  And that's
the end of the quickstart.

[1]: https://generator.cynic-rs.dev
