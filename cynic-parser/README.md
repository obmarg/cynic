<div align="center">
  <img src="https://github.com/obmarg/cynic/raw/main/logo.png" width="150"/>
  <h1>cynic-parser</h1>

  <p>
    <strong>A fast, correct and easy to use GraphQL parser</strong>
  </p>

  <p>
    <a href="https://crates.io/crates/cynic"><img alt="Crate Info" src="https://img.shields.io/crates/v/cynic-parser.svg"/></a>
    <a href="https://docs.rs/cynic-parser/"><img alt="API Docs" src="https://img.shields.io/docsrs/cynic-parser"/></a>
    <a href="https://discord.gg/Y5xDmDP"><img alt="Discord Chat" src="https://img.shields.io/discord/754633560933269544"/></a>
  </p>

  <h4>
    <a href="https://docs.rs/cynic-parser">Documentation</a>
    <span> | </span>
    <a href="https://github.com/obmarg/cynic/tree/main/cynic-parser/examples/examples">Examples</a>
    <span> | </span>
    <a href="https://github.com/obmarg/cynic/blob/main/cynic-parser/CHANGELOG.md">Changelog</a>
  </h4>
</div>

`cynic-parser` is a GraphQL parser - it is part of `cynic` but can also
be used as a standalone parser.

### Design Goals

- Fast to parse
- Fast to compile
- Correct and up to date
- Easy to use
- Minimal memory use

### Features

- Support for parsing executable and type system documents compatible with the
  2021 GraphQl specification or earlier.
- Fancy error reports on failure.
- A prettier compatible pretty printer for GraphQl documents.
