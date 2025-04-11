# Inline Fragments

GraphQL provides interfaces & union types, which are abstract types that can
resolve to one of several concrete objects types. To query these cynic provides
the `InlineFragments` derive.

`InlineFragments` can be derived on an enum with one variant for each sub-type
that you're interested in querying.  Each of the variants should have a single
field containing a type that implements `QueryFragment` for one of the
sub-types.

For example, the GitHub API has an `Assignee` union type which could be queried
with:

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
implement `QueryFragment` for their respective GraphQL types.

#### Fallbacks

Cynic requires a fallback variant on each `InlineFragments` that will be
matched when the server returns a type other than the ones you provide. This
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
struct ActorFallback {
    pub login: String
}
```

This functionality is only available for interfaces as union types have no
concept of shared fields.

##### Fallbacks for unions

If your `InlineFragments` is querying a union your fallback variant can receive
the `__typename` of the type that was received. In the case below, if the
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

#### Exhaustiveness Checking

By default, cynic doesn't implement any kind of exhuastiveness checking on
`InlineFragments`.  This is in line with standard GraphQL behaviour: it's not a
bug to make a query that skips some types that could be returned.

But some users may _want_ exhaustivness checking.  For union types this can be
enabled with the `exhaustive` attribute:

```rust
#[derive(cynic::InlineFragments)]
#[cynic(exhaustive)]
enum Assignee {
    Bot(Bot),
    User(User)

    #[cynic(fallback)]
    Other(String)
}
```

If the a new type is to this union then cynic will fail to compile.  

```admonish warning
This check uses the name of the variants to perform its checks, which is not
fool proof.  There's no guarantee that the `Bot` type actually queries for the
`Bot` type in the server.  So be careful when using this.
```

#### Struct Attributes

An `InlineFragments` can be configured with several attributes on the
enum itself:

- `graphql_type = "AType"` tells cynic which interface or union type
  in the GraphQL schema this enum represents. The name of the enum is
  used if it is omitted.
- `schema` tells cynic which schema to use to validate your InlineFragments.
  The schema you provide should have been registered in your `build.rs`. This
  is optional if you're using the schema that was registered as default, or if
  you're using `schema_path` instead.
- `schema_path` sets a path to the GraphQL schema. This is only required
  if you're using a schema that wasn't registered in `build.rs`.
- `schema_module` tells cynic where to find your schema module. This is
  optional and should only be needed if your schema module is not in scope or
  named `schema`.
- `exhaustive` adds exhaustiveness checking to an `InlineFragment`.  Note that
  this is only supported on GraphQL unions currently (though I would accept a
  PR to add it to interfaces)

#### Variant Attributes

Each variant can also have it's own attributes:

- `fallback` can be applied on a single variant to indicate that it
  should be used whenever cynic encounters a `__typename` that doesn't
  match one of the other variants. For interfaces this can contain a
  `QueryFragment` type. For union types it must be applied on a unit
  variant.
