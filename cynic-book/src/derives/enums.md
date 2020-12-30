# Enums

Much like with query structs cynic expects you to own any enums you want to
query for, or provide as arguments. The `cynic::Enum` trait is used to define
an enum, and the easiest way to define that trait is to derive it:

```rust
#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "Market")]
pub enum Market {
    Uk,
    Ie,
}
```

The derive will work on any enum that only has unit variants that match up with
the variants on the enum in the schema. If there are any extra or missing
variants, the derive will emit errors.

#### Struct Attributes

An Enum can be configured with several attributes on the enum itself:

- `graphql_type = "AType"` is required and tells cynic which type in the
  GraphQL schema to map this enum to
- `rename_all="camelCase"` tells cynic to rename all the rust field names with
  a particular rule to match their GraphQL counterparts. If not provided this
  defaults to `SCREAMING_SNAKE_CASE` to be consistent with GraphQL conventions.

<!-- TODO: list of the rename rules, possibly pulled from codegen docs -->

#### Field Attributes

Each field can also have it's own attributes:

- `rename="SOME_VARIANT"` can be used to map a variant to a completely
  different GraphQL variant name.

<!-- TODO: example of the above?  Better wording -->
