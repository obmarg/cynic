# Changelog

All notable changes to this project will be documented in this file.

The format is roughly based on [Keep a
Changelog](http://keepachangelog.com/en/1.0.0/).

This project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased - xxxx-xx-xx

## v0.9.2 - 2025-08-19

### New Features

- rev function on TypeWrappersIter (#1161)
- add directive support to generator (#1139)

### Bug Fixes

- support `&str` for `String` input fields (#1160)

## v0.9.1 - 2025-02-20

### Bug Fixes

- `TryFrom<Value<'_>> for ConstValue<'_>` now returns an error when an object
  or list contains a `Value`, rather than panicing (#1133)

## v0.9.0 - 2025-02-10

### Breaking Changes

- Added new variants to `cynic_parser::Error` to handle empty documents, these
  errors were previous ambiguous parser errors (#1117)
- `Error::span` now returns an `Option<Span>` instead of a `Span`

### Bug Fixes

- Parsing will no longer fail if keywords like `mutation` are used as enum
  values. (#1128)

### Changes

- Parse schema with `cynic_parser` in querygen (#1124)
- Bumped the MSRV to 1.80

## v0.8.7 - 2024-12-03

### Changes

- Fixed all rust 1.83 clippy lints ([#1106](https://github.com/obmarg/cynic/pull/1106))

## v0.8.6 - 2024-11-28

### Bug Fixes

- `Value::is_variable` now works correctly ([#1104](https://github.com/obmarg/cynic/pull/1104))
- Fixed the `VariableValue` debug impl which was misleading
  ([#1104](https://github.com/obmarg/cynic/pull/1104))

## v0.8.5 - 2024-11-27

### New Features

- Every selection set in `ExecutableDocument` now has a span.

### Bug Fixes

- `Type::wrappers` functions now return `TypeWrappersIter` instead of `impl Iterator`
- `TypeWrappersIter` is now `Clone`

## v0.8.4 - 2024-11-20

### Bug Fixes

- Removed erroneous error code from reports ([#1099](https://github.com/obmarg/cynic/pull/1099))
- Removed some stray dbg! calls ([#1097](https://github.com/obmarg/cynic/pull/1097))

## v0.8.1 - 2024-11-11

### New Features

- Added a span for directive arguments

## v0.8.0 - 2024-11-07

### Breaking Changes

- Lexing failures now include the token that failed to parse
- `Description::raw_str` has been renamed `raw_untrimmed_str` to make its
  purpose clearer.
- `StringLiteral::raw_str` has been renamed `raw_untrimmed_str` to make its
  purpose clearer.
- `Float`s are now correctly represented as f64 rather than f32
- `Value::as_f32` & `ConstValue::as_f32` are now `as_f64`

### New Features

- Added convenience functions to `SchemaDefintion` to get the individual
  definitions from within
- Added `get` function to `ConstList`, `List,`, `ConstObject` and `Object` to
  get the value of an item or field from the list or object
- `ConstList`, `List`, `ConstObject` and `Object` now implement `IntoIterator`,
  which is equivalent to calling `items` or `fields`
- Added `as_f64` & `as_str` & `as_bool` functions to the respective scalar
  value types.

### Bug Fixes

- `Float`s are now correctly represented as f64 rather than f32

### Changes

- `Iter` types are now reexported in the `type_system` and `executable` modules
- `Value` & `ConstValue` are now reexported from the crate root.

## v0.7.1 - 2024-10-25

### Bug Fixes

- Added `InputValueDefinition::default_value_span` again ([#1077](https://github.com/obmarg/cynic/pull/1077))
- Implemented `todo!` in `ConstValue::span` ([#1076](https://github.com/obmarg/cynic/pull/1076))

## v0.7.0 - 2024-10-25

### Breaking Changes

- Removed `InputValueDefiniton::default_value_span` - you should now fetch the
  span from the `default_value` itself.
- `RootOperationTypeDefinition::span` now covers the entire root operation type
  definition. For the old spans, use `RootOperationTypeDefinition::named_type_span`.

### New Features

- Added more spans to the type system AST ([#1070](https://github.com/obmarg/cynic/pull/1070))
- Added more spans to the executable AST ([#1069](https://github.com/obmarg/cynic/pull/1069))
- Added `Type::definitions` for fetching the definitions associated with a type ([#1067](https://github.com/obmarg/cynic/pull/1067))

### Bug Fixes

- Fixed an issue where explicit lifetimes had to be used on
  `Iter<'_, Whatever<'_>>` ([#1072](https://github.com/obmarg/cynic/pull/1072))
- Fixed some associated type definitions on Iter ([#1068](https://github.com/obmarg/cynic/pull/1068))

## v0.6.2 - 2024-10-24

### New Features

- Added `Span::overlaps` function ([#1065](https://github.com/obmarg/cynic/pull/1065))

### Changes

- The IdReader trait now has a lifetime parameter ([#1064](https://github.com/obmarg/cynic/pull/1064))

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
