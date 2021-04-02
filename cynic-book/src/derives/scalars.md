# Scalars

Cynic supports all the built in GraphQL scalars by default. If you want to
query a field of one of these types add a field of the corresponding Rust type
to your `QueryFragment` struct.

- `String` fields in GraphQL should be `String` fields in Rust.
- `Int` fields in GraphQL should be `i32` in Rust.
- `Boolean` fields in GraphQL map to `bool` in Rust.
- `ID` fields in GraphQL map to the `cynic::Id` type in Rust.

## Custom Scalars

### `impl_scalar!`

GraphQL allows a schema to define it's own scalars - cynic also supports these.

If you have an existing type (including 3rd party types) that has a
`serde::Serialize` impl and want to use it as a Scalar, you can use
`impl_scalar!` to register it as a `Scalar`.  For example to register
`chrono::DateTime<chrono::Utc>` as a `DateTime` scalar:

```rust
type DateTime = chrono::DateTime<chrono::Utc>;
impl_scalar!(DateTime, query_dsl::DateTime);
```

This `DateTime` type alias can now be used anywhere that the schema expects a
`DateTime`.  Note that the type alias is currently required due to limitations
in some of the cynic macros (though this may not always be the case).

### `#[derive(Scalar)]`

You can also derive `Scalar` on any newtype structs:

```rust
#[derive(cynic::Scalar, serde::Serialize)]
#[cynic(query_module = "query_dsl")]
struct MyScalar(String);
```

This `MyScalar` type can now be used anywhere the schema expects a `MyScalar`.

Any types that derive `cynic::Scalar` must also derive (or otherwise implement)
`serde::Serialize`.  You can change the inner type that's used to deserialize
the scalar by changing the type inside the struct.

Note that this derive only works on newtype structs - for any more complex
datatype you'll have to implement cynic::Scalar yourself.

#### Struct Attributes

A Scalar derive can be configured with several attributes on the struct itself:

- `graphql_type = "AType"` can be provided if the type of the struct differs
  from the type of and tells cynic the name of the Scalar in the schema.  This
  defaults to the name of the struct if not provided.
  GraphQL schema to map this struct to
- `query_module` tells cynic where to find the query module - that is a module
  that has called the `query_dsl!` macro. This is required but can also be
  provided by nesting the QueryFragment inside a query module.
