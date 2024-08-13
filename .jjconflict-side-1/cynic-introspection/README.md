### Cynic Introspection

`cynic-introspection` defines a [GraphQL introspection query][1] that can be
run using [`cynic`][2], a rust GraphQL client

This can be used for any reason you'd want to introspect a GraphQL server -
including when you're using cynic as a library in your own project.

### Features

- Support for introspecting servers that support both GraphQL 2021 and GraphQL
  2018.
- Contains a capability detection query that can determine which version of the
  GraphQL specification a server supports prior to querying it.

### Usage

[See the documentation on docs.rs for instructions on using
cynic-introspection][3].

[1]: http://spec.graphql.org/June2018/#sec-Introspection
[2]: https://cynic-rs.dev
[3]: https://docs.rs/cynic-introspection
