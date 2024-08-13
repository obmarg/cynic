# Enums

Much like with query structs cynic expects you to own any enums you want to
query for, or provide as arguments. The `cynic::Enum` trait is used to define
an enum. The easiest way to define that trait is to derive it:

```rust
#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum Market {
    Uk,
    Ie,
}
```

The derive will work on any enum that only has unit variants that match up with
the variants on the enum in the schema. If there are any extra or missing
variants, the derive will emit errors.

#### Variant Naming

The GraphQL spec [recommends that enums are "all caps"][1]. To handle this
smoothly, Cynic matches rust variants up to their equivalent
`SCREAMING_SNAKE_CASE` GraphQL variants. This behaviour can be disabled by
specifying a `rename_all = "None"` attribute, or customised alternative
`rename_all` values or individual `rename` attributes on the variants.

#### Exhaustiveness Checking

By default, cynic checks the exhuastiveness of `Enum`s - you should provide a
variant for each enum value in the GraphQL schema.  You can also provide a `fallback` variant to provide forwards compatability - if the server adds new enum values they'll be caught by this variant.

You can opt-out of this exhaustiveness using the `#[cynic(non_exhaustive)]`
attribute.  When this is present exhaustiveness is not checked, and the
fallback variant is used for all the variants missing from the selection.

#### Enum Attributes

An Enum can be configured with several attributes on the enum itself:

- `graphql_type = "AType"` tells cynic which enum in the GraphQL schema this
  enum represents. The name of the enum is used if it is omitted.
- `rename_all="camelCase"` tells cynic to rename all the rust field names with
  a particular rule to match their GraphQL counterparts. If not provided this
  defaults to `SCREAMING_SNAKE_CASE` to be consistent with GraphQL conventions.
- `schema` tells cynic which schema to use to validate this Enum.
  The schema you provide should have been registered in your `build.rs`. This
  is optional if you're using the schema that was registered as default, or if
  you're using `schema_path` instead.
- `schema_path` provides a path to some GraphQL schema SDL. This is only
  required if you're using a schema that wasn't registered in `build.rs`.
- `schema_module` tells cynic where to find your schema module. This is
  optional and should only be needed if your schema module is not in scope or
  is named something other than `schema`.
- `non_exhaustive` can be provided to mark an enum as non-exhaustive.  Such
  enums are required to have a fallback variant, but not required to have
  a variant for each value in the schema.

<!-- TODO: list of the rename rules, possibly pulled from codegen docs -->

#### Variant Attributes

Each variant can also have it's own attributes:

- `rename="SOME_VARIANT"` can be used to map a variant to a completely
  different GraphQL variant name.
- The `fallback` attribute can be provided on a single variant. This variant
  will be used when we receive a value we didn't expect from the server - such
  as when the server has added a new variant since we last pulled its schema.
  This variant can optionally have a single string field, which will receive
  the value we received from the server.

<!-- TODO: example of the above?  Better wording -->

[1]: https://spec.graphql.org/June2018/#sec-Enum-Value
