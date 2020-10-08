# InputObject

Some fields in GraphQL expect you to provide an input object rather than just
simple scalar types. The `cynic::InputObject` trait is used to define an
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
and it may contain scalar values or other `InputObject`s.

By default the field names are expected to match the GraphQL variants
exactly, but this can be controlled with either the `rename_all` top level
attribute or the rename attribute on individual fields.

If there are any fields in the struct that are not on the GraphQL input type
the derive will emit errors. Any required fields that are on the GraphQL input
type but not the rust struct will also error. Optional fields may be omitted
without error. This maintains the same backwards compatability guarantees as
most GraphQL clients: adding a required field is a breaking change, but adding
an optional field is not.

Currently any missing optional fields will not be serialized in queries,
whereas optional fields that are present in the struct but set to None will be
sent as `null`.

<!-- TODO: example of the above?  Better wording. -->

#### Struct Attributes

An InputObject can be configured with several attributes on the struct itself:

- `graphql_type = "AType"` is required and tells cynic which type in the
  GraphQL schema to map this struct to
- `rename_all="camelCase"` tells cynic to rename all the rust field names with a particular
  rule to match their GraphQL counterparts. Typically this would be set to
  `camelCase` but others are supported.
- `require_all_fields` can be provided when you want cynic to make sure your
  struct has all of the fields defined in the GraphQL schema.

#### Field Attributes

Each field can also have it's own attributes:

- `rename="someFieldName"` can be used to map a field to a completely different
  GraphQL field name.
