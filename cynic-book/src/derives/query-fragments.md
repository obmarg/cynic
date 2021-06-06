# Query Fragments

QueryFragments are the main tool for building queries & mutations in cynic. A
QueryFragment tells cynic what fields to select from a GraphQL object, and how
to decode those fields into a struct.

Generally you'll use a derive to create query fragments, like this:

```rust
#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "examples/starwars.schema.graphql",
    schema_module = "schema",
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
    schema_module = "schema",
    graphql_type = "Root",
)]
struct FilmsConnection {
    films: Vec<Film>,
}
```

If the above QueryFragment was used in a query, it would result in GraphQL that
looked like:

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
    schema_module = "schema",
    graphql_type = "Root",
)]
struct AllFilmsQuery {
    all_films: Option<FilmConnection>,
}
```

An `Operation` can be created from this `QueryFragment`:

```rust
use cynic::QueryBuilder;

let operation = AllFilmsQuery::build(());
```

This particular query has no arguments so we provide the unit type `()` in place
of actual arguments.

This `Operation` can be converted into JSON using `serde`, sent to a server, and
then then it's `decode_response` function can be used to decode the response
itself. An example of this is in the [Quickstart][quickstart].

### Passing Arguments

To pass arguments into queries you must pass an `argument_struct` parameter
to the `cynic` attribute, and then add `arguments` attributes to the
fields for which you want to provide arugments. The `argument_struct`
parameter must name a struct that implements `cynic::FragmentArguments`, which
can also be derived. (See [query arguments][1] for more details)

Here, we define a query that fetches a film by a particular ID:

```rust
#[derive(cynic::FragmentArguments)]
struct FilmArguments {
    id: Option<cynic::Id>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "examples/starwars.schema.graphql",
    schema_module = "schema",
    graphql_type = "Root",
    argument_struct = "FilmArguments"
)]
struct FilmQuery {
    #[arguments(id = &args.id)]
    film: Option<Film>,
}
```

This can be converted into a query in a similar way we just need to provide
some `FilmArguments`:

```rust
use cynic::QueryBuilder;

let operation = FilmQuery::build(
    FilmArguments{
        id: Some("ZmlsbXM6MQ==".into()),
    }
);
```

#### Nested Arguments

The example above showed how to pass arguments to the top level of a query. If
you want to pass arguments to a nested QueryFragment then all it's parent
`QueryFragment`s must specify the same `argument_struct` in their `cynic`
attribute. This is neccesary so that the `FragmentArgument` struct gets passed
down to that level of a query.

If no nested QueryFragments require arguments, you can omit the
`argument_struct` attr.

### Mutations

Mutations are also constructed using QueryFragments in a very similar way to
queries. Instead of selecting query fields you select a mutation, and pass in
any arguments in exactly the same way. Mutations use the `MutationBuilder`
rather than `QueryBulder`:

```rust
use cynic::MutationBuilder;

let operation = SomeMutation::build(SomeArguments { ... });
```

This `operation` can then be used in exactly the same way as with queries.

<!-- TODO: An example of doing mutations -->

#### Struct Attributes

A QueryFragment can be configured with several attributes on the struct itself:

- `graphql_type = "AType"` tells cynic which object in the GraphQL schema this
  struct represents. The name of the struct is used if it is omitted.
- `schema_path` sets the path to the GraphQL schema. This is required, but
  can be provided by nesting the QueryFragment inside a query module with this
  attr.
- `schema_module` tells cynic where to find the schema module - that is a module
  module that has called the `use_schema!` macro. This will default to
  `schema` if not provided. An override can also be provided by nesting the
  QueryFragment inside a module with the `schema_for_derives` attribute macro.

#### Field Attributes

Each field can also have it's own attributes:

- `rename = "someGraphqlName"` can be provided if you want the rust field name
  to differ from the GraphQL field name.  You should provide the name as it is
  in the GraphQL schema (although due to implementation details a snake case
  form of the name may work as well)
- `recurse = "5"` tells cynic that this field is recursive and should be
  fetched to a maximum depth of 5. See [Recursive Queries][recursive-queries]
  for more info.
- The `flatten` attr can be used to "flatten" out excessive Options from lists.
  As GraphQL is used in languages with implicit nulls, it's not uncommon to see
  a type `[Int]` - which in Rust maps to `Option<Vec<Option<i32>>`. This isn't
  a very nice type to work with - applying the `flatten` attribute lets you
  represent this as a `Vec<i32>` in your QueryFragment. Any outer nulls become
  an empty list and inner nulls are dropped.
- The `spread` attr can be used to spread another `QueryFragment`s into the
  current `QueryFragment`, if each of the `QueryFragment`s point at the same
  GraphQL type.

### Related

- [FragmentArguments][1] are used to provide arguments to the fields of a
  QueryFragment.
- [Struct Level Attributes][2] can be added to a QueryFragment.
- [Recursive queries][recursive-queries] are supported by QueryFragments.

[1]: ./query-arguments.html
[2]: ../struct-attributes.html
[recursive-queries]: ./recursive-queries.html
[quickstart]: ../quickstart.html
