# Overview

Cynic is a GraphQL library for Rust.  It's not the first but it takes a
different approach from the existing libraries.

Existing libraries take a query first approach to GQL - you write a query using
GraphQL and libraries use that to generate Rust structs for you using macros.
This is really easy and great for getting going quickly.  However, if you want
to use structs that aren't quite what the macros output you're out of luck.
Some more complex use cases like sharing structs among queries are also
commonly not supported.

Cynic takes a different approach - it uses Rust structs to define queries and
generates GraphQL from them.  This gives you freedom to control the structs
you'll be working with while still enjoying type safe queries, checked against
the GraphQL schema.  When it's built in `derives` don't do exactly what you
want it provides lower level APIs to hook in and fetch the data you want in the
format you want to work with it.

Of course writing out all the structs to represent a large GraphQL query can be
quite challenging, and GraphQL has excellent tooling for building queries
usually.  Cynic provides [`querygen`][1] to help with this - you write a
GraphQL query using the existing GQL tooling and it'll generate some cynic
structs to make that query.  You can use this as a starting point for your
projects - either adding on to the rust structs directly, or re-using
`querygen` as appropriate.

### Features

Cynic is currently a work in progress, but the following features are
supported:

- Typesafe queries for scalars, enums & objects.
- Defining custom scalars.
- Building dynamic (but still type checked) queries at run time.
- Query arguments

The following features are possibly supported but not very thoroughly tested:

- Mutations.
- Sending input objects as arguments to queries or mutations.
- Fetching union types via inline fragments

The following features are not yet supported, though hopefully will be someday
(please open an issue if you'd like to implement them yourself)

- Fetching interface types.
- GraphQL subscriptions.
- Potentially other things (please open an issue if you find anything obviously
  missing)

### Documentation

Cynic is documented in a few places:

1. The guide that you're reading on [cynic-rs.dev](https://cynic-rs.dev)
2. The reference documentation on [docs.rs](https://docs.rs/cynic)

### Using This Guide

If you're new to Cynic the [quickstart](./quickstart.html) is a good place to
start.  Afterwards you might want to read the [derives](./derives/) chapter,
for more details about how to do common things with Cynic.

The [Building Queries](./building-queries/) section is for more advanced users
- either you've run into a case that the built in derives don't cover, or
you're just curious how things work under the hood.

### Inspiration

- [graphql-client][2], the original Rust GraphQL client.  This is a great
  library for using GraphQL from Rust. It wasn't quite what I wanted but it
  might be what you want.
- The idea of encoding the GraphQL typesystem into a DSL was taken from
  [elm-graphql][3].
- Most of the JSON decoding APIs were taken from [Json.Decode in Elm][4].
- Deriving code from structs is a fairly common Rust pattern, though [serde][5]
  in particular provided inspiration for the derive APIs.

[1]: https://generator.cynic-rs.dev
[2]: https://github.com/graphql-rust/graphql-client
[3]: https://github.com/dillonkearns/elm-graphql
[4]: https://package.elm-lang.org/packages/elm/json/latest/Json.Decode
[5]: https://serde.rs
