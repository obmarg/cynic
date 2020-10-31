# Recursive Queries

GraphQL allows for types to recurse and allows queries of those recursive types
to a particular depth. Cynic supports this in it's `QueryFragment` derive when
users provide the `recurse` attribute on the field that recurses.

If we wanted to recurisvely fetch characters that were in a star wars film and the other films that they were in (and so on) we could do:

```rust
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Film")]
struct Film {
    title: Option<String>,
    character_connection: Option<CharacterConnection>,
}


#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "FilmCharacterConnection")]
struct CharacterConnection {
    characters: Option<Vec<Option<CharacterConnection>>>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Person")]
struct Character {
    name: Option<String>,
    films: Option<PersonFilmsConnection>
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Person")]
struct PersonFilmsConnection {
    #[cynic(recurse = "5")]
    films: Option<Vec<Option<Film>>>
}
```

The `#[cynic(recurse = "5")]` attribute on `films` in `PersonFilmsConnection`
lets cynic know that it should recursively fetch films to a maximum depth of 5.

### Recursive Field Types

When the `recurse` attribute is present on a field, the rust type that field is
required to be may change from a non-recursive field.

- If the field is not nullable it will need to be wrapped in `Option` to
  account for the case where we finish recursing.
- If the field is nullable but not inside a list, it's normal `Option` will
  need to be wrapped in a `Box`, as this is required for recursive types in
  Rust.
- If the field is not nullable and not inside a list, it will also need a
  `Box`. This box goes inside the `Option`.

Some examples of the changes:

| GraphQL Type | Rust Type Without Recurse | Rust Type With Recurse   |
| ------------ | ------------------------- | ------------------------ |
| `T`          | `Option<T>`               | `Box<Option<T>>`         |
| `T!`         | `T`                       | `Option<Box<T>>`         |
| `[T]`        | `Option<Vec<Option<T>>>`  | `Option<Vec<Option<T>>>` |
| `[T]!`       | `Vec<Option<T>>`          | `Option<Vec<Option<T>>>` |
| `[T!]`       | `Option<Vec<T>`           | `Option<Vec<T>>`         |
