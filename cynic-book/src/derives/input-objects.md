# InputObject

Some fields in GraphQL expect you to provide an input object rather than just
simple scalar types.  The `cynic::InputObject` trait is used to define an
input object and the easiest way to define that trait is to derive it:

```rust
#[derive(cynic::InputObject, Clone, Debug)]
#[cynic(graphql_type = "IssueOrder", rename_all = "camelCase")]
pub struct IssueOrder {
    pub direction: OrderDirection,
    pub field: IssueOrderField,
}
```

The derive will work on any struct that matches the format of the input object
- it may contain scalar values or other `InputObject`s. If there are any extra
or missing fields the derive will emit errors.

By default the field names are expected to match the GraphQL variants
exactly, but this can be controlled with either the `rename_all` top level
attribute or the rename attribute on individual fields.

<!-- TODO: example of the above?  Better wording.  Detailed docs on attrs -->
