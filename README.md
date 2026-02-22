<div align="center">
  <img src="https://github.com/obmarg/cynic/raw/main/logo.png" width="150"/>
  <h1>Cynic</h1>

  <p>
    <strong>A bring your own types GraphQL client for Rust</strong>
  </p>

  <p>
    <a href="https://crates.io/crates/cynic"><img alt="Crate Info" src="https://img.shields.io/crates/v/cynic.svg"/></a>
    <a href="https://docs.rs/cynic/"><img alt="API Docs" src="https://img.shields.io/badge/docs.rs-cynic-green"/></a>
    <a href="https://web.libera.chat/#cynic"><img alt="IRC Room" src="https://img.shields.io/badge/Chat%20Room-5555FF"/></a>
  </p>

  <h4>
    <a href="https://cynic-rs.dev">Documentation</a>
    <span> | </span>
    <a href="https://github.com/obmarg/cynic/tree/main/examples/examples">Examples</a>
    <span> | </span>
    <a href="https://github.com/obmarg/cynic/blob/main/CHANGELOG.md">Changelog</a>
  </h4>
</div>

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
the GraphQL schema. When its built in `derives` don't do exactly what you
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
- Introspection via [`cynic-cli`][6] or [`cynic-introspection`][7]
- GraphQL Subscriptions via [`graphql-ws-client`][8].
- Field directives (`@skip`, `@include` and any custom directives that don't
  require client support)

### Documentation

Cynic is documented in a few places:

1. There's a guide to using cynic on [cynic-rs.dev](https://cynic-rs.dev)
2. The reference documentation on [docs.rs](https://docs.rs/cynic)

### Inspiration

- [graphql-client][2], the original Rust GraphQL client. This is a great
  library for using GraphQL from Rust. It wasn't quite what I wanted but it
  might be what you want.
- The idea of encoding the GraphQL typesystem into a DSL was taken from
  [elm-graphql][3].
- Most of the JSON decoding APIs were taken from [Json.Decode in Elm][4].
- Deriving code from structs is a fairly common Rust pattern, though [serde][5]
  in particular provided inspiration for the derive APIs.

### Getting Help

If you want help with cynic you can join the #cynic chat room on
IRC using any of these options:

- [A Web UI](https://web.libera.chat/#cynic)
- [Using this link if you have an IRC client](ircs://irc.libera.chat:6697/#cynic)
- [Selecting a client from joinrc.at](https://joinirc.at/ircs://irc.libera.chat:6697/#cynic)
- Joining #cynic on irc.libera.chat if you know what you're doing

[1]: https://generator.cynic-rs.dev
[2]: https://github.com/graphql-rust/graphql-client
[3]: https://github.com/dillonkearns/elm-graphql
[4]: https://package.elm-lang.org/packages/elm/json/latest/Json.Decode
[5]: https://serde.rs
[6]: https://crates.io/crates/cynic-cli
[7]: https://crates.io/crates/cynic-introspection
[8]: https://github.com/obmarg/graphql-ws-client
