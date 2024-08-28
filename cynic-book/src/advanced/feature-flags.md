# Feature Flagged Queries

Often in GraphQL there's only one server for a given schema.  But sometimes
there might be many servers that serve a different schema, and those servers
might be serving different versions of that same schema. A classic example of
this is the introspection query: servers support a different set of fields
depending on which version of the specification they support and/or which RFCs
they have implemented on top of that specification.

To support these cases, cynic allows you to associate fields in a
`QueryFragment` with a feature:

```rust
#[derive(cynic::QueryFragment, Debug)]
struct AuthorQuery {
    __typename: String,
    #[cynic(feature = "shiny")]
    shiny_new_field: Option<String>
}

```

This `Author` struct will only query for `shiny_new_field` if the `shiny`
feature has been enabled.

To enable features you need to use the lower-level `OperationBuilder` to build
your `Operation` (rather than the `QueryBuilder` that is usually recommended):

```rust
let operation = cynic::OperationBuilder::<QueryWithFeatures, ()>::query()
    .with_variables(())
    .with_feature_enabled("shiny")
    .build()?;
```

For examples of this feature in action, take a look in the
`cynic-introspection` crate, which uses it to support multiple versions of the
GraphQL specification, and the various RFCs that servers support.
