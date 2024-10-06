# Changelog

All notable changes to this project will be documented in this file.

The format is roughly based on [Keep a
Changelog](http://keepachangelog.com/en/1.0.0/).

This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased - xxxx-xx-xx

## v0.6.1 - 2024-10-06

### Bug Fixes

- Fixed some issues with block string trimming ([#1060](https://github.com/obmarg/cynic/pull/1060))

## v0.6.0 - 2024-10-03

### New Features

- Rework `Value` significantly ([#1048](https://github.com/obmarg/cynic/pull/1048))
- Added `ConstValue` ([#1057](https://github.com/obmarg/cynic/pull/1057))
- The ExecutableId & TypeSystemId traits have been changed ([#1047](https://github.com/obmarg/cynic/pull/1047))

### Changes

- MSRV is now 1.76

## v0.5.2 - 2024-09-25

### New Features

- `Description` is now Display
  ([#1044](https://github.com/obmarg/cynic/pull/1044))

### Bug Fixes

- `Iter::with_ids` now takes self by reference, which works better with
  `Iter` no longer being `Copy`
  ([#1045](https://github.com/obmarg/cynic/pull/1045))

## v0.5.1 - 2024-09-25

### Bug Fixes

- Added the `Description` convenience functions that were missed in v0.5.0
  ([#1041](https://github.com/obmarg/cynic/pull/1041))

## v0.5.0 - 2024-09-25

### Breaking Changes

- Integer values are now represented by `IntValue` rather than an i32.
  Although the GraphQl `Int` type is represented by `i32` this restriction is
  not specified for the grammar.
  ([#1037](https://github.com/obmarg/cynic/pull/1037))
- `Iter` is no longer `Copy` as this could cause subtle bugs. It remains
  `Clone` so you can use that if you need to.
  ([#1036](https://github.com/obmarg/cynic/pull/1036))
- `IdRange` now implements `IntoIterator` instead of directly implementing
  `Iterator`. ([#1036](https://github.com/obmarg/cynic/pull/1036))

### New Features

- Pretty printer can now optionally sort definitions & fields in its output
  ([#1038](https://github.com/obmarg/cynic/pull/1038))
- The type ystem AST now has spans in more places
  ([#998](https://github.com/obmarg/cynic/pull/998/files)).

### Bug Fixes

- Support ints larger than i32 in parser ([#1037](https://github.com/obmarg/cynic/pull/1037))
- use Iter in more places in parser ([#1030](https://github.com/obmarg/cynic/pull/1030))

## v0.4.5 - 2024-06-19

### New Features

- Add `Iter::with_ids` for iterating over readers and their corresponding `Id`s
  ([#984](https://github.com/obmarg/cynic/pull/984))

## v0.4.4 - 2024-06-10

### New Features

- impl Ord for parser id types ([#981](https://github.com/obmarg/cynic/pull/981))

## v0.4.3 - 2024-06-04

### Bug Fixes

- `FragmentSpread::fragment` had a missing lifetime
  ([#978](https://github.com/obmarg/cynic/pull/978))

## v0.4.2 - 2024-06-04

### New Features

- Added `FragmentSpread::fragment` function that looks up the named fragment
  ([#976](https://github.com/obmarg/cynic/pull/976))

## v0.4.1 - 2024-05-31

### New Features

- Added `Value::variables_used` to find variables used in a value
  ([#963](https://github.com/obmarg/cynic/pull/963))

## v0.4.0 - 2024-05-2

### Breaking Changes

- Pretty printing has been moved behind a new feature flag `pretty`
- The names of the pretty printing functions have been updated with a `_pretty`
  prefix.

### New Features

- All of the readers in the executable module now impl Display, allowing you
  to use them with `print!` and friends. This is hidden behind the `print`
  feature. ([#962](https://github.com/obmarg/cynic/pull/962))
- All of the `Id` types now impl `Hash`, `PartialEq`, `Eq` & `Debug`
  ([#961](https://github.com/obmarg/cynic/pull/961))
- All of the readers now have an `id` function that allows you to retreive an
  `Id` for that reader. ([#959](https://github.com/obmarg/cynic/pull/959))
- `Iter` now exposes a function `ids` that allows you to retrieve the underlying
  `IdRange` ([#959](https://github.com/obmarg/cynic/pull/959))

### Bug Fixes

- Fixed a lot of bad formatting in the pretty printing of schema documents
  ([#957](https://github.com/obmarg/cynic/pull/957))
- Pretty printing will now add whitespace between fields & arguments that have
  a docstring ([#954](https://github.com/obmarg/cynic/pull/954))

### Changes

- update rust crate logos to 0.14 ([#942](https://github.com/obmarg/cynic/pull/942))

## v0.2.7 - 2024-05-14

### New Features

- Updated most functions to return the named type `Iter` rather than
  `impl ExactSizedIterator` ([#945](https://github.com/obmarg/cynic/pull/945))
- All the readers now `impl Debug`
  ([#923](https://github.com/obmarg/cynic/pull/923))

### Bug Fixes

- Parser now errors on an invalid directive location instead of panicing
  ([#948](https://github.com/obmarg/cynic/pull/948))
- Pretty printing now formats long arguments correctly
  ([#947](https://github.com/obmarg/cynic/pull/947))

## [v0.2.6](https://github.com/obmarg/cynic/compare/cynic-parser-v0.2.5...cynic-parser-v0.2.6) - 2024-04-16

### Features

- implement PartialEq for Type and Value ([#924](https://github.com/obmarg/cynic/pull/924))

## [v0.2.5](https://github.com/obmarg/cynic/compare/cynic-parser-v0.2.4...cynic-parser-v0.2.5) - 2024-04-15

### Fixes

- Directives on schema definitions are now parsed correctly
  ([#921](https://github.com/obmarg/cynic/pull/921))
- Schema extensions without operation definitions are now parsed correctly
  ([#921](https://github.com/obmarg/cynic/pull/921))

## [v0.2.4](https://github.com/obmarg/cynic/compare/cynic-parser-v0.2.3...cynic-parser-v0.2.4) - 2024-04-15

### Fixes

- TypeSystemDocument::definitions() now returns an ExactSizeIterator
  ([#919](https://github.com/obmarg/cynic/pull/919))

## [0.2.3](https://github.com/obmarg/cynic/compare/cynic-parser-v0.2.2...cynic-parser-v0.2.3) - 2024-04-15

### Fixes

- Pretty printing output is now significantly improved, but still not perfect
  ([#916](https://github.com/obmarg/cynic/pull/916))

## v0.2.2 - 2024-04-12

### Fixes

- Pretty printing will now escape strings

## v0.2.1 - 2024-04-04

### Changes

- Some internal changes

## v0.2.0 - 2024-03-25

### Features

- Added executable parsing support
- String escaping is now supported properly.

### Changes

- Probably a ton of other things. I'm not going to list the changes
  exhaustively because of how early this library is - I'd be surprised
  if anyone was using it (if I'm wrong please let me know).

## v0.1.0 - 2024-01-23

- Initial version.
- Supports parsing & printing GraphQL schemas
- Is fast
