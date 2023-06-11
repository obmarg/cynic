# Query Fragments

QueryFragments are the main tool for building queries & mutations in cynic.
Cynic builds up a GraphQL query document from the fields on a `QueryFragment`
and any `QueryFragments` nested inside it. And after executing an operation it
deserializes the result into the `QueryFragment` struct.

Generally you'll use a derive to create query fragments, like this:

```rust
#[derive(cynic::QueryFragment, Debug)]
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
of a schema can be used to build a `cynic::Operation`. This `Operation` is the
type that should be serialized and sent to the server.

If we wanted to use our FilmConnection to get all the films from the star wars
API we need a QueryFragment like this:

```rust
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Root")]
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

This `Operation` can be serialized into JSON using `serde`, sent to a server,
and then then a `cynic::GraphQlResponse<AllFilmsQuery>` can be deserialized
from the response. An example of this is in the [Quickstart][quickstart].

### Passing Arguments

GraphQL allows a server to define arguments that a field can accept. Cynic
provides support for passing in these arguments via its `arguments` attribute.

Here, we define a query that fetches a film by a particular ID:

```rust
#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Root")]
struct FilmQuery {
    #[arguments(id: "ZmlsbXM6MQ==")]
    film: Option<Film>,
}
```

Note the `#[arguments: id: "ZmlsbXM6MQ=="]` attribute on the `film` field. The
GraphQL generated for this query will provide a hard coded `id` argument to the
`film` field, like this:

```graphql
film(id: "ZmlsbXM6MQ==") {
  title
  director
}
```

The syntax of the inside of arguments is very similar to [the syntax expected
for arguments in GraphQL itself][gql-arguments]. Some examples:

| GraphQL                        | Cynic                          |
| ------------------------------ | ------------------------------ |
| `input: { filters: "ACTIVE" }` | `input: { filters: "ACTIVE" }` |
| `values: ["Hello"]`            | `values: ["Hello"]`            |
| `values: ["Hello"]`            | `values: ["Hello"]`            |
| `arg1: "Foo", arg2: "Bar"`     | `arg1: "Foo", arg2: "Bar"`     |
| `arg1: null                    | `arg1: null`                   |

### Variables

If you don't want to hard code the value of an argument, you can parameterise
your query with some variables. These variables must be defined on a struct:

```rust
#[derive(cynic::QueryVariables)]
struct FilmQueryVariables {
    id: Option<cynic::Id>,
}
```

The fields of this struct can be any `Enum`, `InputObject`, or `Scalar`.

To use this struct you need to tell your `QueryFragment` that it takes variables
using the `variables` parameter to to the `cynic` attribute, and then you can
use variables much like you would in GraphQL.

Here, we update our `FilmQuery` struct to make use of our `FilmQueryVariables`
to provide the `id` argument.

```rust
#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Root",
    variables = "FilmQueryVariables"
)]
struct FilmQuery {
    #[arguments(id: $id)]
    film: Option<Film>,
}
```

Any field of the variables struct may be used by prefixing the name of the
field with `$`.

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

See [query variables][1] for more details.

#### Nested Variables

The example above showed how to pass variables to the top level of a query. If
you want to pass variables to a nested QueryFragment then all it's parent
`QueryFragment`s must specify the same `variables` in their `cynic`
attribute. This is necessary so that the `QueryVariables` struct gets passed
down to that level of a query.

If no nested `QueryFragments` require arguments, you can omit the
`variables` attr from those `QueryFragments`

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
- `variables` defines the `QueryVariables` struct that is available to
  `arguments` attributes on fields of the given struct.
- `schema` tells cynic which schema to use to validate your InlineFragments.
  The schema you provide should have been registered in your `build.rs`.  This
  is optional if you're using the schema that was registered as default, or if
  you're using `schema_path` instead.
- `schema_path` sets a path to the GraphQL schema. This is only required
  if you're using a schema that wasn't registered in `build.rs`.
- `schema_module` tells cynic where to find your schema module.  This is
  optional and should only be needed if your schema module is not in scope or
  named `schema`.

#### Field Attributes

Each field can also have it's own attributes:

- `rename = "someGraphqlName"` can be provided if you want the rust field name
  to differ from the GraphQL field name. You should provide the name as it is
  in the GraphQL schema (although due to implementation details a snake case
  form of the name may work as well)
- `alias` can be provided if you have a renamed field and want to explicitly
  request a GraphQL alias in the resulting query output.
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

- [QueryVariables][1] are used to provide variables to a QueryFragment.
- [Recursive queries][recursive-queries] are supported by QueryFragments.

[1]: ./query-variables.html
[recursive-queries]: ./recursive-queries.html
[quickstart]: ../quickstart.html
[gql-arguments]: https://graphql.org/learn/queries/#arguments
