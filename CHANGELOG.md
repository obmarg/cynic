# Changelog

All notable changes to this project will be documented in this file.

The format is roughly based on [Keep a
Changelog](http://keepachangelog.com/en/1.0.0/).

This project intends to inhere to [Semantic
Versioning](http://semver.org/spec/v2.0.0.html), but has not yet reached 1.0 so
all APIs might be changed.

## Unreleased - xxxx-xx-xx

### New Features

- Union types can be queried via `#[derive(InlineFragments)]` on an enum.
- Schemas that use interfaces are now supported, though interfaces are not
  yet queryable.
- `#[derive(QueryFragment)]` now explicitly checks for required/list type
  mismatches & other easy mistakes, and warns the user appropriately.
- Added `scalars_as_strings` for stubbing out all scalar types as strings.
  A temporary measure until I can come up with an easier way to manage large
  numbers of scalars.
- Added an `output_query_dsl` function suitable for running inside build.rs

### Changed

- Split the procedural macros out into their own cynic-proc-macros crate.
  cynic-codegen now exists as a re-usable library for programatically 
  doing the codegen.
- The IntoArguments trait is now named FromArguments and has had it's
  parameters switched up.
- Added a StarWars API example

### Bug Fixes

- Now supports schemas that define their root query types.  Before we just
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
