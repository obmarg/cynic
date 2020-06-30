# Scalars

Cynic supports all the built in GraphQL scalars by default.  If you want to
query a field of one of these types add a field of the corresponding Rust type
to your `QueryFragment` struct.

- `String` fields in GraphQL should be `String` fields in Rust.
- `Int` fields in GraphQL should be `i64` in Rust (though this will be changing
  to i32 soon, to align with the GraphQL spec).
- `Boolean` fields in GraphQL map to `bool` in Rust.
- `ID` fields in GraphQL map to the `cynic::Id` type in Rust.

### Custom Scalars

GraphQL allows a schema to define it's own scalars - cynic also supports these.
You can implement the `Scalar` trait manually, but it's recommended to use a derive:

```
#[derive(cynic::Scalar)]
struct MyScalar(String);
```

This defines a scalar called MyScalar - use this in a `QueryFragment` where you
want to fetch a field of type `MyScalar` (which serializes to a String).

You can change the inner type that's used to serialize & deserialize the scalar
by changing the type inside the struct.

Note that this derive only works on newtype structs - for any more complex
datatype you'll have to implement cynic::Scalar yourself.
