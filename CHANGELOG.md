# Changelog

All notable changes to this project will be documented in this file.

The format is roughly based on [Keep a
Changelog](http://keepachangelog.com/en/1.0.0/).

This project intends to inhere to [Semantic
Versioning](http://semver.org/spec/v2.0.0.html), but has not yet reached 1.0 so
all APIs might be changed.

## Unreleased - xxxx-xx-xx

### New Features

- Added chrono::DateTime scalar support behind a chrono feature flag.
- The `cynic::selection_set` module and all it's contents are now documented.

### Bug Fixes

- Generated `query_dsl` now disables unused import warnings where appropriate.
- Exposed `Id::inner` & `Id::into_inner` functions - these were meant to be
  public but were not
- Cleaned up a ton of compiler warnings - mostly unused imports and a few unused
  variables
- `query_dsl` adds `allow(dead_code)` annotations so we don't get tons of dead
  code warnings when we're not exercising an entire schema.
- `query_dsl` no longer creates mutable `Vec` for fields without arguments -
  this was leading to tons of "doesn't need to be mutable" warnings.

## v0.2.0 - 2016-06-11

### Breaking Changes

- The generated `query_dsl` no longer contains generated enums - users should
  provide their own enums and `derive(cynic::Enum)` on them. Cynic querygen
  can be used to help with this.
- The generated `query_dsl` no longer contains generated input objects - users
  should provide their own structs and `impl cynic::InputObject` on them. A
  derive for this should be coming in the future.

### New Features

- Union types can be queried via `#[derive(InlineFragments)]` on an enum.
- Schemas that use interfaces are now supported, though interfaces are not
  yet queryable.
- `#[derive(QueryFragment)]` now explicitly checks for required/list type
  mismatches & other easy mistakes, and warns the user appropriately.
- Added an `output_query_dsl` function suitable for running inside build.rs
- Added a flatten option at the field level. When present this will flatten
  nested options & vectors into the provided type. Used to handle the common
  case in GQL where someone has defined an optional list of optionals. This is
  a pain in Rust, since the same thing can usually be represented by a
  non-optional list of non-optionals.
- Added a cynic-querygen for generating QueryFragment structs from a graphql
  schema & query. This currently has a WIP web interface and a WIP CLI, though
  neither of them are particularly user friendly at this point.
- Added a `cynic::query_module` attribute macro that can be applied to modules
  containing QueryFragments & InlineFragments. When this attribute is present
  the derive will be done for all QueryFragments & InlineFragments contained
  within. This allows users to omit some of the parameters these derives
  usually require, as the `query_module` attribute provides them and fills them
  in. These modules may be expanded in the future to provide more
  "intelligent" features.
- Added support for mapN up to N = 50, therefore adding support for GraphQL
  objects with up to 50 fields.
- Added new `cynic::Enum` derive that matches up a Rust enum with a GraphQL enum.
  `cynic-querygen` will automatically provide enums using this derive when a
  query includes an enum.
- Added `SelectionSet::and_then` for chaining decode operations on selection sets.
- Added `cynic::Scalar` derive for newtype structs so that users can easily
  define their own scalars.  Also added support for this to cynic-querygen
- Added `cynic::Id` type to handle Ids in queries.
- Added `cynic::InputObject` trait to allow the `query_dsl` to handle
  InputObjects generically.

### Changed

- Split the procedural macros out into their own cynic-proc-macros crate.
  cynic-codegen now exists as a re-usable library for programatically
  doing the codegen.
- The IntoArguments trait is now named FromArguments and has had it's
  parameters switched up.
- Added a StarWars API example

### Bug Fixes

- Now supports schemas that define their root query types. Before we just
  assumed there was a type called query.
- Fixed a few things that stop the examples in the documentation from
  compiling.
- Fixed all the tests
- We now use the correct case for non built-in scalar types
- Fixed an issue that prevented propagation of argument structs into inner
  QueryFragments

## v0.1.2 - 2020-02-04

- No changes

## v0.1.1 - 2020-02-04

- Some tweaks to the documentation.

## v0.1.0 - 2020-02-03

- Initial release
