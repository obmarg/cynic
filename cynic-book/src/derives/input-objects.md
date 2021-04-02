# InputObject

Some fields in GraphQL expect you to provide an input object rather than just
simple scalar types. The `cynic::InputObject` trait is used to define an
input object and the easiest way to define that trait is to derive it:

```rust
#[derive(cynic::InputObject, Clone, Debug)]
pub struct IssueOrder {
    pub direction: OrderDirection,
    pub field: IssueOrderField,
}
```

The derive will work on any struct that matches the format of the input object
and it may contain scalar values or other `InputObject`s. See `Field Naming`
below for how names are matched between GraphQL & Rust.

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

### Field Naming

It's a common GraphQL convention for fields to be named in `camelCase`. To
handle this smoothly, Cynic matches rust fields up to their equivalent
`SCREAMING_SNAKE_CASE` GraphQL fields. This behaviour can be disabled by
specifying a `rename_all = "None"` attribute, or customised alternative
`rename_all` values or individual `rename` attributes on the fields.

#### Struct Attributes

An InputObject can be configured with several attributes on the struct itself:

- `graphql_type = "AType"` tells cynic which input object in the GraphQL
  schema this struct represents. The name of the struct is used if it is omitted.
- `require_all_fields` can be provided when you want cynic to make sure your
  struct has all of the fields defined in the GraphQL schema.
- `rename_all="camelCase"` tells cynic to rename all the rust field names with
  a particular rule to match their GraphQL counterparts. If not provided this
  defaults to camelCase to be consistent with GraphQL conventions.

<!-- TODO: list of the rename rules, possibly pulled from codegen docs -->

#### Field Attributes

Each field can also have it's own attributes:

- `rename="someFieldName"` can be used to map a field to a completely
  different GraphQL field name.
- `skip_serializing_if="path"` can be used on optional fields to skip
  serializing them. By default an `Option` field will be sent as `null` to
  servers, but if you provide `skip_serializing_if="Option::is_none"` then the
  field will not be provided at all.
