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
enum Assignee {
    Bot(Bot),
    Mannequin(Mannequin)
    Organization(Organization),
    User(User)

    #[cynic(fallback)]
    Other
}
```

Where each of `Bot`, `Mannequin`, `Organization` & `User` are all structs that
implement `QueryFragment` for the respective GraphQL types.

#### Fallbacks

Cynic requires a fallback variant on each `InlineFragments` that will be
matched when the server returns a type other than the ones you provide.  This
allows your code to continue compiling & running in the face of additions to
the server, similar to the usual GraphQL backwards compatibility guarantees.

```rust
#[derive(cynic::InlineFragments)]
enum Assignee {
    Bot(Bot),
    User(User)

    #[cynic(fallback)]
    Other
}
```

##### Fallbacks for interfaces

If your `InlineFragments` is querying an interface your fallback variant can
also select some fields from the interface:

```rust
#[derive(cynic::InlineFragments, Debug)]
pub enum Actor {
    User(User),

    #[cynic(fallback)]
    Other(ActorFallback),
}

#[derive(cynic::QueryFragment)]
enum ActorFallback {
    pub login: String
}
```

This functionality is only available for interfaces as union types have no
concept of shared fields.

##### Fallbacks for unions

If your `InlineFragments` is querying a union your fallback variant can receive
the `__typename` of the type that was received.  In the case below, if the
`Assignee` is not a `Bot` or a `User`, the `String` field of `Other` will be
populated with the name of the type (the `__typename`) that was actually
received from the server.

```rust
#[derive(cynic::InlineFragments)]
enum Assignee {
    Bot(Bot),
    User(User)

    #[cynic(fallback)]
    Other(String)
}
```

This functionality is currently only available for unions.

#### Struct Attributes

An `InlineFragments` can be configured with several attributes on the
enum itself:

- `graphql_type = "AType"` tells cynic which interface or union type
  in the GraphQL schema this enum represents. The name of the enum is
  used if it is omitted.
- `schema` tells cynic which schema to use to validate your InlineFragments.
  The schema you provide should have been registered in your `build.rs`.  This
  is optional if you're using the schema that was registered as default, or if
  you're using `schema_path` instead.
- `schema_path` sets a path to the GraphQL schema. This is only required
  if you're using a schema that wasn't registered in `build.rs`.
- `schema_module` tells cynic where to find your schema module.  This is
  optional and should only be needed if your schema module is not in scope or
  named `schema`.

#### Variant Attributes

Each variant can also have it's own attributes:

- `fallback` can be applied on a single variant to indicate that it
  should be used whenever cynic encounters a `__typename` that doesn't
  match one of the other variants. For interfaces this can contain a
  `QueryFragment` type. For union types it must be applied on a unit
  variant.
