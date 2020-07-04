# Inline Fragments

InlineFragments are used when querying an interface or union type, to determine
which of the sub-types the query returned and to get at the fields inside that
type.

`InlineFragments` can be derived on any enum that's variants contain a single
type that implements `QueryFragment`.  Each of the `QueryFragment`s should have
a `graphql_type` that maps to one of the sub-types of the `graphql_type` type
for the `InlineFragment`.

For example, the GitHub API has an `Assignee` union type which could be queried with:

```rust
#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "github.graphql",
    query_module = "query_dsl",
    graphql_type = "Assignee"
)]
enum Assignee {
    Bot(Bot),
    Mannequin(Mannequin)
    Organization(Organization),
    User(User)
}
```

Where each of `Bot`, `Mannequin`, `Organization` & `User` are all structs that
implement `QueryFragment` for the respective GraphQL types.
