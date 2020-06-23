# Quickstart

If you just want to get going with cynic and don't care too much about how it works, this is the chapter for you.

#### Pre-requisites

There's a few things you'll need before you get started:

1. An existing rust project (though you can just run `cargo new` if you don't have one).
2. A GraphQL API that you'd like to query, and a copy of it's schema.
3. A GraphQL query that you'd like to run against the API. If you don't have one of these, you should probably use graphiql or graphql playground to get started.

#### Adding cynic as a dependency.

First things first: you need to add cynic to your dependencies. Open up your `Cargo.toml` and add the following under the `[dependencies]` section:

```toml
cynic = "0.6"
```

TODO: Probably want to mention insta & surf?

Run a `cargo check` to make sure this builds and you're good to go.

TODO: Copying schema into build

#### Building your query structs.

Cynic allows you to build queries from Rust structs - so you'll need to take the query you're wanting to run, and convert it into one or more rust structs. This can be quite
laborious and error prone for larger queries though so cynic provides [`querygen`][1] to help you get started.

Go to [https://generator.cynic-rs.dev] (please excuse the UI) and enter your schema (in GraphQL SDL) and your GraphQL query in the left most 2 textareas. The right most text area should now contain some Rust code. You should copy and paste this into a file in your repository.

TODO: Example w/ simple starwars query

#### Checking your query

TODO: Write a test case w/ insta to check the query output.

#### Making your query

TODO: Write the code for making a query

[1]: https://generator.cynic-rs.dev
