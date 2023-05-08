# Schemas

GraphQL APIs have a schema that defines the data they can provide.  Cynic uses
this schema to verify that the queries & mutations you're writing are valid and
typesafe.

## Pre-registering Schemas

The easiest way to provide schemas to the cynic derives is to register them in
your `build.rs`, with some code similar to the following:

```rust
    cynic_codegen::register_schema("github")
        .from_sdl_file("schemas/github.graphql")
        .unwrap()
        .as_default()
        .unwrap();
```

This will register a schema called `github` as well as registering it as the
default schema.  This schema will now automatically be used by any cynic derive
in your crate.

## Schema Modules

The cynic derives also require a "schema module" to be in scope.  If you've
already pre-registered your schema then defining this looks like:

```cynic
#[cynic::schema("starwars")
mod schema {}
```

## Working with Multiple schemas

If you need to work with multiple APIs with different schemas, simply give them
different names when registering.  Here we add the starwars schema as well:

```rust
    cynic_codegen::register_schema("starwars")
        .from_sdl_file("schemas/starwars.graphql")
        .unwrap();
```

Note how we didn't call `.as_default()` here - the `github` schema remains our
default, but we can also use `starwars` when we need to. To use this in a
derive you'll have to provide the `schema` argument:

```
#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema = "starwars")]
struct Film {
    title: Option<String>,
    director: Option<String>,
}
```

You'll also need to make sure you define a schema module for each of your
schemas, and make it available as `schema` in the files where you're defining
your queries.
