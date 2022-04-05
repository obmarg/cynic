# Variables

A hierarchy of QueryFragments can take a struct of variables. This struct must
implement `QueryVariables` which can be derived:

```rust
#[derive(cynic::QueryVariables)]
struct FilmVariables {
    id: Option<cynic::Id>,
}
```

### Using QueryVariables

To use any fields of this struct as an argument to a QueryFragment, the struct
must provide a `variables` parameter that points to the `FilmArguments`
struct. This allows variables to be passed in using the `arguments`
attribute on the fields where you wish to pass them.

```rust
#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "examples/starwars.schema.graphql",
    graphql_type = "Root",
    variables = "FilmVariables"
)]
struct FilmQuery {
    #[arguments(id: $id)]
    film: Option<Film>,
}
```

This example uses our `FilmVariables` at the root of the query to specify which
film we want to fetch.

It's also possible to pass variables down to lower levels of the query using
the same technique. Though it's worth noting that all the QueryFragments from
the Root to the point that requires arguments must define the same
`variables` in their cynic attribute. If no nested QueryFragments
require any variables then it's OK to omit `variables`.
