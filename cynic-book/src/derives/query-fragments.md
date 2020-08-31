# Query Fragments

QueryFragments are the main tool for building queries in cynic. A
QueryFragment tells cynic what fields to select from a GraphQL object, and how
to decode those fields into a struct.

Generally you'll use a derive to create query fragments, like this:

```rust
#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "examples/starwars.schema.graphql",
    query_module = "query_dsl",
    graphql_type = "Film"
)]
struct Film {
    title: Option<String>,
    director: Option<String>,
}
```

When this struct is used in a query it will select the `title` & `director`
fields of the `Film` type, which are both optional strings. QueryFragments can
be nested inside each other, like so:

```rust
#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "examples/starwars.schema.graphql",
    query_module = "query_dsl",
    graphql_type = "Root",
)]
struct FilmsConnection {
    films: Vec<Film>,
}
```

If the above QueryFragment was used in a query, it would result in GraphQL that looked like:

```
films {
  title
  director
}
```

QueryFragments are compile time checked against the provided GraphQL schema.
You cannot nest a `Film` QueryFragment into a field that was expecting an
`Actor` for example. Similarly, nullable fields must be an `Option` and lists
must be a `Vec`.

<!-- TODO: Could maybe put an example error in here? -->

### Making a Query with QueryFragments

QueryFragments that apply to the Query type (otherwise known as the Root type)
of a schema can be used to build a `cynic::Query`. This is the type that can
be sent to a server and used to decode the response.

If we wanted to use our FilmConnection to get all the films from the star wars
API we need a QueryFragment like this:

```rust
#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "examples/starwars.schema.graphql",
    query_module = "query_dsl",
    graphql_type = "Root",
)]
struct AllFilmsQuery {
    all_films: Option<FilmConnection>,
}
```

This can be used as a query like so:

```
let query = cynic::Operation::query(AllFilmsQuery::fragment(());
```

This `Query` can be converted into JSON using `serde`, sent to a server, and
then then it's `decode_response` function can be used to decode the response
itself. An example of this is in the [Quickstart][quickstart].

The empty `()` we pass to fragment is for the arguments - this particular query
has no arguments so we pass `Void`.

### Passing Arguments

To pass arguments into queries you must pass an `argument_struct` parameter
to the `cynic` attribute, and then add `arguments` attributes to the
fields for which you want to provide arugments. The `argument_struct`
parameter must name a struct that implements `cynic::FragmentArguments`, which
can also be derived. (See [query arguments][1] for more details)

Here, we define a query that fetches a film by a particular ID:

```rust
#[derive(Clone, cynic::FragmentArguments)]
struct FilmArguments {
    id: Option<cynic::Id>,
}

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

This can be converted into a query in a similar way we just need to provide
the arguments:

```rust
let query = cynic::Operation::query(FilmQuery::fragment(FilmArguments{
    id: Some("ZmlsbXM6MQ==".into()),
}));
```

#### Nested Arguments

The example above showed how to pass arguments to the top level of a query. If
you want to pass arguments to a nested QueryFragment then all it's parent
`QueryFragment`s must specify the same `argument_struct` in their `cynic`
attribute. This is neccesary so that the `FragmentArgument` struct gets passed
down to that level of a query.

If no nested QueryFragments require arguments, you can omit the
`argument_struct` attr.

### Related

- [FragmentArguments][1] are used to provide arguments to the fields of a
  QueryFragment.
- [Struct Level Attributes][2] can be added to a QueryFragment.

[1]: ./query-arguments.html
[2]: ../struct-attributes.html
[quickstart]: ../quickstart.html
