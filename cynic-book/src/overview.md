<h1>Cynic</h1>

<p>
  <strong>A bring your own types GraphQL client for Rust</strong>
</p>

<p>
  <a href="https://crates.io/crates/cynic"><img alt="Crate Info" src="https://img.shields.io/crates/v/cynic.svg"/></a>
  <a href="https://docs.rs/cynic/"><img alt="API Docs" src="https://img.shields.io/badge/docs.rs-cynic-green"/></a>
  <a href="https://discord.gg/Y5xDmDP"><img alt="Discord Chat" src="https://img.shields.io/discord/754633560933269544"/></a>
  <!-- 
      <a href="https://blog.rust-lang.org/2020/07/16/Rust-1.45.0.html"><img alt="Rustc Version 1.45+" src="https://img.shields.io/badge/rustc-1.45%2B-lightgrey.svg"/></a>
      -->
</p>

<h4>
  <a href="https://github.com/obmarg/cynic">Repository</a>
  <span> | </span>
  <a href="https://github.com/obmarg/cynic/tree/master/examples/examples">Examples</a>
  <span> | </span>
  <a href="https://github.com/obmarg/cynic/blob/master/CHANGELOG.md">Changelog</a>
</h4>

# Overview

Cynic is a GraphQL library for Rust. It's not the first but it takes a
different approach from the existing libraries.

Existing libraries take a query first approach to GQL - you write a query using
GraphQL and libraries use that to generate Rust structs for you using macros.
This is really easy and great for getting going quickly. However, if you want
to use structs that aren't quite what the macros output you're out of luck.
Some more complex use cases like sharing structs among queries are also
commonly not supported.

Cynic takes a different approach - it uses Rust structs to define queries and
generates GraphQL from them. This gives you freedom to control the structs
you'll be working with while still enjoying type safe queries, checked against
the GraphQL schema. When it's built in `derives` don't do exactly what you
want it provides lower level APIs to hook in and fetch the data you want in the
format you want to work with it.

Of course writing out all the structs to represent a large GraphQL query can be
quite challenging, and GraphQL has excellent tooling for building queries
usually. Cynic provides [`querygen`][1] to help with this - you write a
GraphQL query using the existing GQL tooling and it'll generate some cynic
structs to make that query. You can use this as a starting point for your
projects - either adding on to the rust structs directly, or re-using
`querygen` as appropriate.

### Features

Cynic is currently a work in progress, but the following features are
supported:

- Typesafe queries & mutations.
- Defining custom scalars.
- Building dynamic (but still type checked) queries at run time.
- Query arguments including input objects
- Interfaces & union types
- GraphQL Subscriptions via [`graphql-ws-client`](https://github.com/obmarg/graphql-ws-client).
  (though `graphql-ws-client` is fairly alpha quality)

The following features are not yet supported, though should be soon (if you
want to help out with the project I'd be happy for someone else to try and
implement these - if you open an issue I'd be happy to give pointers on how to
go about implementing any of them)

- Directives
- Potentially other things (please open an issue if you find anything obviously
  missing)

### Documentation

Cynic is documented in a few places:

1. The guide that you're reading on [cynic-rs.dev](https://cynic-rs.dev)
2. The reference documentation on [docs.rs](https://docs.rs/cynic)

### Using This Guide

If you're new to Cynic the [quickstart](./quickstart.html) is a good place to
start. Afterwards you might want to read the [derives](./derives/index.html)
chapter, for more details about how to do common things with Cynic.

The [Building Queries](./building-queries/index.html) section is for more
advanced users - either you've run into a case that the built in derives
don't cover, or you're just curious how things work under the hood.

### Inspiration

- [graphql-client][2], the original Rust GraphQL client. This is a great
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
