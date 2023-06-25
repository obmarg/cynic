<div align="center">
  <img src="https://github.com/obmarg/cynic/raw/main/logo.png" width="150"/>
  <h1>cynic-cli</h1>

  <p>
    <strong>A CLI for Cynic, the code first GraphQL client for Rust</strong>
  </p>

  <p>
    <a href="https://crates.io/crates/cynic"><img alt="Crate Info" src="https://img.shields.io/crates/v/cynic.svg"/></a>
    <a href="https://docs.rs/cynic/"><img alt="API Docs" src="https://img.shields.io/badge/docs.rs-cynic-green"/></a>
    <a href="https://discord.gg/Y5xDmDP"><img alt="Discord Chat" src="https://img.shields.io/discord/754633560933269544"/></a>
  </p>

  <h4>
    <a href="https://cynic-rs.dev">Documentation</a>
    <span> | </span>
    <a href="https://github.com/obmarg/cynic/blob/main/CHANGELOG.md">Changelog</a>
  </h4>
</div>

# Overview

Cynic is a code first GraphQL client for Rust. `cynic-cli` is
a CLI that provides utilities for working with `cynic`
specifically and GraphQL in general.

## Features

- Can introspect a remote server and dump it's schema.
- Intelligent feature detection for introspection - checks which features a 
  server supports and only introspects for those.

## Usage

```console
$ cynic help
A CLI for cynic, a code first GraphQL client for Rust

Usage: cynic [COMMAND]

Commands:
  introspect  Runs an introspection query against a GraphQL server and outputs the servers schema
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

### Introspect

The `introspect` command runs an introspection query against a server and
prints the schema to stdout/a file.

```console
$ cynic help introspect
Runs an introspection query against a GraphQL server and outputs the servers schema

Usage: cynic introspect [OPTIONS] <URL>

Arguments:
  <URL>
          The URL of the GraphQL schema that we should introspect

Options:
  -H, --header <HEADERS>
          Any headers to send with the introspection request
          
          These should be in HTTP format e.g. `-H "Authorization: Bearer a_token_123"`

  -o, --output <OUTPUT>
          The name of a file we should output the schema into.
          
          By default we print to stdout.

      --server-version <SERVER_VERSION>
          The version of the GraphQL specificaiton that the remote GraphQL server implements
          
          Different versions of GraphQL expose different fields via introspection, so we need to know which set of fields to ask for.
          
          By default we run an additional query to figure out what the server we're talking to supports.
          
          [default: auto]

          Possible values:
          - 2018: Run an introspection query compatible with the 2018 GraphQL specification
          - 2021: Run an introspection query compatible with the 2021 GraphQL specification
          - auto: Run an additional query to determine what the GraphQL server supports

  -h, --help
          Print help (see a summary with '-h')

```

