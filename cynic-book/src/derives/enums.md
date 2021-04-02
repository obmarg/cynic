# Enums

Much like with query structs cynic expects you to own any enums you want to
query for, or provide as arguments. The `cynic::Enum` trait is used to define
an enum, and the easiest way to define that trait is to derive it:

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

#### Enum Attributes

An Enum can be configured with several attributes on the enum itself:

- `graphql_type = "AType"` tells cynic which enum in the GraphQL schema this
  enum represents. The name of the enum is used if it is omitted.
- `rename_all="camelCase"` tells cynic to rename all the rust field names with
  a particular rule to match their GraphQL counterparts. If not provided this
  defaults to `SCREAMING_SNAKE_CASE` to be consistent with GraphQL conventions.

<!-- TODO: list of the rename rules, possibly pulled from codegen docs -->

#### Variant Attributes

Each variant can also have it's own attributes:

- `rename="SOME_VARIANT"` can be used to map a variant to a completely
  different GraphQL variant name.

<!-- TODO: example of the above?  Better wording -->

1: https://spec.graphql.org/June2018/#sec-Enum-Value
