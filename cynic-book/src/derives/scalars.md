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
`impl_scalar!` to register it as a `Scalar`. For example to register
`chrono::DateTime<chrono::Utc>` as a `DateTime` scalar:

```rust
use chrono::{DateTime, Utc};
impl_scalar!(DateTime<Utc>, schema::DateTime);
```

You can now use a `DateTime<Utc>` type for any `DateTime` in your scheam.

```admonish info
This `impl_scalar` call should be placed in the crate that defines the the
`schema` module.
```

### `#[derive(Scalar)]`

You can also derive `Scalar` on any newtype structs:

```rust
#[derive(cynic::Scalar, serde::Serialize)]
struct MyScalar(String);
```

This `MyScalar` type can now be used anywhere the schema expects a `MyScalar`.

Any types that derive `cynic::Scalar` must also derive (or otherwise implement)
`serde::Serialize`. You can change the inner type that's used to deserialize
the scalar by changing the type inside the struct.

```admonish info
This derive only works on newtype structs - for any more complex datatype
you'll have to implement cynic::Scalar yourself, or use `impl_scalar` above
```

#### Struct Attributes

A Scalar derive can be configured with several attributes on the struct itself:

- `graphql_type = "AType"` can be provided if the type of the struct differs
  from the type of and tells cynic the name of the Scalar in the schema. This
  defaults to the name of the struct if not provided.
- `schema_module` tells cynic where to find your schema module. This is
  optional and should only be needed if your schema module is not in scope or
  named `schema`.
