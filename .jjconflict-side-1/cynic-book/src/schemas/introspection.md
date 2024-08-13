# Introspecting an API

Each GraphQL server has a schema that describes the information it can provide.
Cynic needs a local copy of the schema for any APIs that it needs to talk to.

This page documents various ways to get a copy of that schema and keep it up to
date.

## Introspecting with the `cynic` CLI

The easiest way to introspect a schema is with the `cynic` CLI.

You can install the CLI with `cargo`:
```sh
cargo install --locked cynic-cli
```

The CLI has an `introspect` subcommand that can be used to introspect a server
and output it's schema to a file.  The format for this command is:

```sh
cynic introspect [GRAPHQL_URL] -o [OUTPUT_FILE]
```

~~~admonish example
To fetch the StarWars API schema we use in the documentation and
put it in `schemas/starwars.graphql`:

```sh
cynic introspect https://swapi-graphql.netlify.app/.netlify/functions/index -o schemas/starwars.graphql
```
~~~

### Providing Headers

Some GraphQL APIs require headers to introspect (e.g. for authentication).  The
introspection command supports these via the `-H` parameter, which expects a
header in HTTP format.

~~~admonish example
The GitHub API sometimes requires an `Authorization` header which can be provided with `-H`:

```sh
cynic introspect -H "Authorization: Bearer [GITHUB_TOKEN]" "https://graphql.org/swapi-graphql" -o schemas/github.graphql
```
~~~

## Keeping the schema up to date

When using cynic, we recommend you keep a copy of the remote schema checked
into your repository.  This means you can always build your application from
source without having to download the schema.

Some schemas might not be updated particularly often, so you might run
introspection once when starting your project and be done.  

But if you want to keep your local copy of the schema up to date, I'd recommend
adding a period CI job that will fetch the schema and make a commit with the up
to date schema, running your normal CI process to make sure there were no
breakages.

If you're using GitHub Actions you could use the following as a template:

```yaml
name: Update local copy of GraphQL schema
on: 
  schedule:
    - cron: 0 9 * * *
jobs:
  update-schema:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write
    steps:
    - uses: actions/checkout@v3
    - name: Run introspection
      uses: obmarg/action-graphql-introspect@main
      with:
        server-url: https://swapi-graphql.netlify.app/.netlify/functions/index
        output-file: schemas/starwars.graphql
    - name: Create Pull Request
      uses: peter-evans/create-pull-request@v5
      with:
        commit-message: "Update StarWars GraphQL Schema"
        branch: graphql-schema-updates
        title: "Update StarWars GraphQL Schema"
        message: "This is an automated pull request to update our local schema cache"
```

```admonish warn
This action requires that you've given GitHub actions permission to open a PR.  See [the create-pull-request action documentation for more details][1]
```

## Introspecting in Code

You may also want to run an introspection query in code, either to integrate
with cynic or if you just need an introspection query for other reasons.  The
[`cynic-introspection`][2] crate can do this for you, please refer to [its
documentation][3].

[1]: https://github.com/peter-evans/create-pull-request#workflow-permissions
[2]: https://crates.io/crates/cynic-introspection
[3]: https://docs.rs/cynic-introspection
