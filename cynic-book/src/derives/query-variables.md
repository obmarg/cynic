# Variables

GraphQL queries can declare variables that can be passed in, allowing you to
set the values of arguments without specifying those values directly in your
query. You can use this to pass values from the rest of your program into
cynic.

You can declare a set of variables by making a struct and deriving
`QueryVariables` on it:

```rust
#[derive(cynic::QueryVariables)]
struct FilmVariables {
    id: Option<cynic::Id>,
}
```

The struct above declares a single variable named `id` of `Option<cynic::Id>`
(or `ID` in GraphQL terms).

### Using QueryVariables

To use variables in a query you need to tell cynic which `QueryVariables`
struct is in scope.  You do this by providing a a `variables` parameter to the
`QueryFragment` derive.  This allows you to provide any of the variables in your
`QueryVariables` struct to any arguments in this fragment.  For example:

```rust
#[derive(cynic::QueryFragment, Debug)]
#[cynic(
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
