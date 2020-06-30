# Enums

Much like with query structs cynic expects you to own any enums you want to
query for, or provide as arguments.  The `cynic::Enum` trait controls is used
to define an enum, and the easiest way to define that trait is to derive it:

```rust
#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "Market")]
pub enum Market {
    UK,
    IE,
}
```

The derive will work on any enum that only has unit variants that match up with
the variants on the enum in the schema.  If there are any extra or missing
variants, the derive will emit errors.

By default the variant names are expected to match the GraphQL variants
exactly, but this can be controlled with either the `rename_all` top level
parametr or the rename variant parameter.

<!-- TODO: example of the above?  Better wording -->
