# Query Arguments

A hierarchy of QueryFragments can take a struct of arguments. This struct must
implement `FragmentArguments` which can be derived:

```
#[derive(Clone, cynic::FragmentArguments)]
struct FilmArguments {
    id: Option<cynic::Id>,
}
```

This derive can be used on any struct containing any fields - the fields do not
need to be specifically related to GraphQL or used in a query, though if you
don't use them at all you should get dead code warnings from Rust.

### Using FragmentArguments

To use any fields of this struct as an argument to a QueryFragment, the struct
must provide an `argument_struct` parameter that points to the `FilmArguments`
struct. This allows arguments to be passed in using the `arguments`
attribute on the fields where you wish to pass them.

```rust
#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "examples/starwars.schema.graphql",
    query_module = "query_dsl",
    graphql_type = "Root",
    argument_struct = "FilmArguments"
)]
struct FilmQuery {
    #[arguments(id = &args.id)]
    film: Option<Film>,
}
```

This example uses our `FilmArguments` at the root of the query to specify which
film we want to fetch.

It's also possible to pass arguments down to lower levels of the query using
the same technique. Though it's worth noting that all the QueryFragments from
the Root to the point that requires arguments must define the same
`argument_struct` in their cynic attribute. If no nested QueryFragments
require any arguments then it's OK to omit `argument_struct`.

### IntoArguments

Cynic uses the `IntoArguments` trait to convert arguments into the correct type.
You can provide your own definition of this trait, but built in conversions are
provided for:

1. Converting bare scalars & enums into Options. This means you don't have to
   explicitly wrap an argument in `Some`. This also allows cynic to be tolerant
   of schemas changing a required argument into an optional argument (which
   would usually be considered a non-breaking change when your client in a
   dynamic language)
2. Converting references to scalars & enums into owned arguments via `clone`.
   Cynic doesn't currently support taking arguments by reference, but this
   convenience saves users from having to explicitly clone.
