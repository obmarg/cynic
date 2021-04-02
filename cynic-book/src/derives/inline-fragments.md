# Inline Fragments

InlineFragments are used when querying an interface or union type, to determine
which of the sub-types the query returned and to get at the fields inside that
type.

`InlineFragments` can be derived on any enum that's variants contain a single
type that implements `QueryFragment`. Each of the `QueryFragment`s should have
a `graphql_type` that maps to one of the sub-types of the `graphql_type` type
for the `InlineFragment`.

For example, the GitHub API has an `Assignee` union type which could be queried with:

```rust
#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "github.graphql",
    query_module = "query_dsl",
)]
enum Assignee {
    Bot(Bot),
    Mannequin(Mannequin)
    Organization(Organization),
    User(User)
}
```

Where each of `Bot`, `Mannequin`, `Organization` & `User` are all structs that
implement `QueryFragment` for the respective GraphQL types.

#### Fallbacks

By default cynic will error you if you leave out any possible type for a given
union type of interface. If you don't want to provide cases for each of the
possible types you can provide the `fallback` attribute on a variant. That
variant will be output whenever an unhandled type is returned.

```rust
#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "github.graphql",
    query_module = "query_dsl",
)]
enum Assignee {
    Bot(Bot),
    User(User)

    #[cynic(fallback)]
    Other
}
```

A fallback can also be provided when you have handled all cases - this will
allow your code to continue to compile even in the face of server changes.

##### Fallbacks for interfaces

If your `InlineFragments` is querying an interface your fallback variant can
also select some fields from the interface:

```rust
#[derive(cynic::InlineFragments, Debug)]
#[cynic(
    schema_path = "github.graphql",
    query_module = "query_dsl",
)]
pub enum Actor {
    User(User),

    #[cynic(fallback)]
    Other(ActorFallback),
}

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "github.graphql",
    query_module = "query_dsl",
)]
enum ActorFallback {
    pub login: String
}
```

This functionality is only available for interfaces as union types have no
concept of shared fields.

#### Struct Attributes

An `InlineFragments` can be configured with several attributes on the
enum itself:

- `graphql_type = "AType"` tells cynic which interface or union type
  in the GraphQL schema this enum represents. The name of the enum is
  used if it is omitted.
- `schema_path` sets the path to the GraphQL schema. This is required,
  but
  can be provided by nesting the InlineFragments inside a query module
  with this attr.
- `query_module` tells cynic where to find the query module - that is a
  module that has called the `query_dsl!` macro. This is required but
  can also be provided by nesting the QueryFragment inside a query
  module.

#### Variant Attributes

Each variant can also have it's own attributes:

- `fallback` can be applied on a single variant to indicate that it
  should be used whenver cynic encounters a `__typename` that doesn't
  match one of the other variants. For interfaces this can contain a
  `QueryFragment` type. For union types it must be applied on a unit
  variant.
