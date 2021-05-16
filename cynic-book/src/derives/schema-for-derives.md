# Schema For Derives

There are a couple of common attributes that need to be provided to most of the cynic derives:

- `schema_path` - the file path to the schema you're working with.
- `query_module` - the rust path to the module in which you.

You can provide these attributes manually on each of the derived structs you
write. But if you've got a lot of structs this might be tedious and/or just a
lot of noise.

To work around this, cynic provides the `schema_for_derives` macro. This macro can
be applied to any `mod` and will populate the `schema_path` & `query_module`
attributes of any cynic derives contained within.

For example this module contains a single QueryFragment and inherits it's
`schema_path` & `query_module` parameters from that outer `schema_for_derives` attribute:

```rust
#[cynic::schema_for_derives(
    file = r#"schema.graphql"#,
    module = "schema",
)]
mod queries {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query")]
    pub struct PullRequestTitles {
        #[arguments(name = "cynic".into(), owner = "obmarg".into())]
        pub repository: Option<Repository>,
    }
}
```
