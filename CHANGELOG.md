# Changelog

All notable changes to this project will be documented in this file.

The format is roughly based on [Keep a
Changelog](http://keepachangelog.com/en/1.0.0/).

This project intends to inhere to [Semantic
Versioning](http://semver.org/spec/v2.0.0.html), but has not yet reached 1.0 so
all APIs might be changed.

## Unreleased - xxxx-xx-xx

## v0.14.0 - 2021-06-06

### New Features

- You can now `spread` a `QueryFragment` into another `QueryFragment` with the
  `#[cynic(spread)]` field attribute.
- The `QueryFragment` derive now supports renaming fields.

### Bug Fixes

- Underscore field names are now supported in schemas and for querying.
- Field names with leading underscores will no longer have those leading
  underscores removed.

## v0.13.2 - 2021-05-16

This release is only of the `cynic` crate - other crates remain at 0.13.1

### Bug Fixes

- This fixes a problem with JSON decoding that made it extremely inefficient
  (particularly on larger responses).  In my benchmarking this improves
  decoding performance 10x.

## v0.13.1 - 2021-04-26

### Bug Fixes

- Fixes an issue where cynic would incorrectly case module certain names in the
  `use_schema` output

## v0.13.0 - 2021-04-04

### Breaking Changes

There are a number of breaking changes here, though they shouldn't require too
much work for users to update. An example of an upgrade can be found
[here](https://github.com/obmarg/git-lead-time/pull/1).

- The `cynic::Scalar` derive has some new requirements:
  - You should now derive (or otherwise implement) `serde::Serialize` for your
    Scalar types.
  - The derive now requires a `schema_module` parameter. The
    `schema_for_derives` macro will automatically insert this (if you're using
    it) but you may need to add `use super::schema;` to ensure it's
    in-scope.
  - The derive now has an optional `graphql_type` parameter. This is required
    if the name of your type and the name of the scalar in the schema differ.
- Scalars have been revamped:
  - The scalar trait now has a typelock. This means that a Scalar impl is now
    directly tied to the scalar definition in a given `query_dsl`.
  - As a result, cynic can no longer define generic Scalar impls for 3rd party
    types (such as `chrono`, `url`, `uuid` etc.). The `impl_scalar` macro has
    been provided to allow users to use these types (or any type that is
    `serde::Serialize`) in their queries.
  - Cynic no longer requires you to define Scalar types you are not using.
  - `select` functions generated for scalar fields in `query_dsl` now take
    a selection_set that decodes the scalar type. This gives some flexibility
    around scalar types.
  - `query_dsl` now defines markers for all the scalar types. As such you
    should not import any custom scalars into your query_dsl module.
- Required scalar arguments no longer have concrete types, so anything
  that relied on type inference (i.e. `arg = "hello".into()`) will no longer
  work. You should either call an explicit function, or rely on a `InputType`
  impl to do the conversion.
- The `uuid`, `chrono`, `bson`, and `url` features have been retired. If you
  were using these you should register them as `Scalar` with the `impl_scalar!`
  macro.
- `SerializableArgument` has been retired in favour of just using
  `serde::Serialize`.
- The return type of `cynic::Enum::select` now includes the `TypeLock` of the
  enum. This should only affect users that were implementing `cynic::Enum`
  directly. Users of the derive should be unaffected.
- `IntoArgument` has been removed in favour of the new `InputType` trait.
- `cynic::SerializeError` no longer exists.

### New Features

- Support for building and decoding subscription queries.
- Alpha quality support for subscriptions over websockets with
  [`graphql-ws-client`](https://github.com/obmarg/graphql-ws-client).

### Deprecated

- `GraphQLError`, `GraphQLResponse`, & `GraphQLResult` have all been deprecated
  in favour of `GraphQlError`, `GraphQlResponse`, & `GraphQlResult`. The types are
  otherwise the same, just with different names.
- `query_dsl!` has been deprecated in favour of a similar macro named `use_schema!`
- It's now recommended that you name your `query_dsl` module `schema` instead.
- `cynic_codegen::output_query_dsl` is now named `cynic_codegen::output_schema_module`.
- The `query_module` attribute macro has been deprecated in favour of
  `schema_for_derives`
- The `query_module` parameter to the derives has been deprecated in favour of
  `schema_module`.

### Bug Fixes

- Cynic will now fail to compile you when you use an incorrect enum type for a
  field in a QueryFragment.
- Field type mismatch errors in QueryFragments are now reported on the span of
  the field type.
- You no longer need to define Scalars you are not using
- If a server adds a new scalar it will no longer break cynic clients.
- Fixed a case where InputObject errors were being shown against the
  `query_module` attr of the module they were defined in.

### Changes

- The `graphql_type` parameter for most of the derives is now optional. It
  defaults to the name of the struct/enum if not present.
- Cynic will no longer generate invalid QueryFragments if fields are not
  selected on a composite.
- Cynic will now error if a QueryFragment selects no fields.

## v0.12.3 - 2021-04-01

## Bug Fixes

- The `cynic::Scalar` derive output no longer contains a spurious `?` that
  clippy warns about in Rust 1.51.

## v0.12.2 - 2021-02-22

### Bug Fixes

- Hopefully fixed the build of documentation for docs.rs

## v0.12.1 - 2021-02-12

### Bug Fixes

- `chrono::NaiveTime` now supports times without seconds (e.g. `10:30`)
- Failures to parse chrono types now provide error messages that mention
  chrono: these were fairly hard to diagnose before.
- The generator no longer prints out an empty types module when there are
  no custom scalars.

## v0.12.0 - 2021-02-08

### Breaking Changes

- `selection_set::inline_fragments` now takes a backup selection set parameter
  for when we get an unexpected `__typename`.
- The `InlineFragments` derive now performs exhaustiveness checking and
  validates the provided variants.
- `InlineFragments` has a new required function `fallback`

### New Features

- `InlineFragments` are now validated for exhaustiveness & correctness.
- `InlineFragments` now support a fallback variant for the case where users
  only care about some of the possibilities.
- `QueryFragment` can now be derived for interfaces
- `chrono::NaiveTime` is now supported as a Scalar when the `chrono` feature is
  active
- Cynic errors will now suggest possible fixes when you mis-spell or mis-name
  a type in your code.

### Changes

- Updated reqwest dependency to 0.11

## v0.11.1 - 2021-01-30

### Bug Fixes

- Optional InputObject arguments can now be provided by reference. Previously
  this required a clone.
- The generator no longer panics if it can't find the root type of a schema.
- The generator no longer tries (and fails) to run queries when it has no URL
  to work with.
- Makes sure docs.rs builds documentation for the HTTP client code.

## v0.11.0 - 2020-12-31

### Breaking Changes

- `QueryFragment::fragment` now accepts a `FragmentContext` rather than
  arguments. This change was neccesary to support recursive queries.
- It is no longer recommended to use `QueryFragment::fragment` directly -
  instead an Operation should be constructed with `QueryBuilder::build` or
  `MutationBuilder::build`. Note that these return an `Operation` so you no
  longer need to construct one manually.
- GraphQL fields with names that match rust keywords will no longer be
  postfixed with a `_` - you should now use a raw identifier (or rename) for
  these fields.
- Derived `Enums` now default to `rename_all = "SCREAMING_SNAKE_CASE"` if not
  provided. To revert to the old behaviour you should explicitly provide "None"
- Derived `InputObjects` now default to `rename_all = "camelCase"` if not
  provided. To revert to the old behaviour you should explicitly provide
  `rename_all = "None"`
- Removed the `kebab-case` & `SCREAMING-KEBAB-CASE` `rename_all` rules - fairly
  sure kebab case is not valid in GraphQL so this shouldn't affect much.

### New Features

- Cynic now supports recursive queries via the `#[cynic(recurse="N")]`
  attribute on fields that recurse.
- The generator now understands query fragments, spreads and inline fragment
  spreads. Inline fragments for interface/union types are not yet supported.
- Interfaces can be queried via `#[derive(InlineFragments)]` on an enum.
- Added support for using chrono::NaiveDate as scalars. The decode/encode
  functions will convert to/from dates in the ISO 8601 format, that is
  `YYYY-MM-DD`
- Added `QueryBuilder` & `MutationBuilder` traits for constructing Operations
  from QueryFragments.

### Bug Fixes

- The generator (and therefore the generator tests) should now work when run on
  windows.
- Paths output as part of generator are now all raw strings, so should support
  windows path separators.
- The generator now correctly wraps literal ID parameters with `cynic::Id::New`
- Cynic derives (and cynic itself) no longer emit clippy warnings (on 1.48 at least)
- We now support raw identifiers in QueryFragment derives, useful for fields
  named `type` or similar.
- The generator now correctly renames InputObject fields & Enum variants if the
  default `rename_all` doesn't work for them.
- The `InputObject` derive no longer looks up scalars inside `query_dsl` (which
  required them to be `pub use`d in `query_dsl`).
- The generator is now context aware with argument values, and does a better job
  of figuring out whether to clone or take by reference.

### Changes

- The generator is now a lot more thorough, it:
  - Deduplicates generated types
  - Supports multiple queries, sharing structs between all the generated queries
    as appropriate.
  - Generates unique names for each struct it creates, even when faced with
    different structs targetting the same type.
  - Generates partial InputObjects when faced with literals with missing fields
    (previously it would generate all fields even when unused)
  - Correctly generates different argument structs if a single type with
    arguments is used in multiple queries. Though the correct IntoArgument impl
    is not yet generated.
- The generator now generates scalars with public fields
- The generator now derives `Clone` on scalars as certain positions they can
  appear in require cloning
- The generator now outputs correct Rust code when faced with queries that rely
  on list coercion.
- `rename_all` attribute is no longer case sensitive.
- Improved the docs for attributes in the book

## v0.10.1 - 2020-11-04

## Bug Fixes

- Implemented SerializableArgument for the various scalars in the
  `integrations` folder. This was preventing them actually being used as
  scalars in input contexts.

## v0.10.0 - 2020-10-11

### Breaking Changes

- `QueryFragment::fragment` and `InlineFragment::fragments` now accept their
  Argument parameters by reference.
- `define_into_argument_for_scalar` has been renamed to
  `impl_into_argument_for_options`
- There's no longer a blanket impl of `SerializableArgument` for any `Scalar`.
  `SerializableArgument` now needs to be implemented on each `Scalar`. There's
  a `impl_serializable_argument_for_scalar` macro that does this. The `Scalar`
  derive automatically calls this macro, so this is only a change if you have a
  custom `Scalar`
- The `IntoArgument` trait now has an `Output` associated type that is used for
  the return value of `IntoArgument::into_argument`
- The `InputObject` derive no longer complains if you omit optional fields.
  The old behaviour can be brought back by attaching a `require_all_fields`
  annotation to the InputObject.
- SerializeError now requires Send + Sync on it's boxed value.

### New Features

- The `bson` feature, which allows to use ObjectId in schemas, added.
- The `uuid` feature, which allows to use Uuid in schemas, added.
- The `url` feature, which allows to use Url in schemas, added.
- `InputObject`s may now contain fields inside a `Box`. This allows for
  recursive `InputObject` types.
- The `surf` feature enables integration with the `surf` HTTP client, so users
  don't have to write it themselves.
- The `reqwest` & `reqwest-blocking` features, which add support for the
  `reqwest` HTTP client.
- Optional fields on an InputObject may now be annotated with
  `skip_serializing_if="path"`, similar to serde. This allows users to omit
  fields from InputObjects under certain circumstances.
- All optional fields of the GraphQLError type are now modeled according to the
  spec, including the `extensions` field, which is expressed as an
  `Option<serde_json::Value>`.

### Changes

- A `SerializableArgument` no longer needs to be `'static + Send`.
- `FragmentArguments` & `InputObject`s no longer need to be `Clone`.

## v0.9.0 - 2020-09-11

### Breaking Changes

- InputObject no longer has a serialize method - this is now handled by a
  SerializableArgument impl instead, which is generated by the InputObject
  derive.
- `Query` has been renamed to `Operation` to make it clear it's used for both
  queries & mutations.
- `Query::new` is now `Operation::query`

### New Features

- InputObjects can now be derived and will be generated by querygen.
- Querygen output is now tested more thoroughly - should be less changes
  required by users just to get it to compile.
- Cynic now supports running & generating code for mutations.

### Removed Features

- Removed the `optimised_query_modules` feature from codegen, as it invovled
  more code than it was worth to keep it around. Functionally this should make
  no difference, though it may change performance characteristics of compiling
  cynic code. Didn't seem to make a significant difference when I was using it
  though.

### Bug Fixes

- Fixed a compile issue in the generated `query_dsl` for schemas with fields
  with > 1 required argument.
- Fixed an issue that required users to add `serde_json` to their dependencies.
  We now re-export it as `cynic::serde_json` and use that in our derive output.
- querygen now adds `rename_all="SCREAMING_SNAKE_CASE"` to Enums by default -
  the GQL convention is to have them in this format and querygen was already
  doing the transformation into the `PascalCase` rust usually uses so this
  should make things more likely to work by default.
- Removed fontawesome from the querygen HTML. Think I added this along with
  bulma but it's not being used, and adds 400kb to the payload.
- Fixed a bug where querygen would not snake case field names when generating
  `QueryFragment`s.
- querygen will now take references to arguments rather than ownership (which
  didn't work for most non-enum types).
- Fixed an issue where querygen was adding ID literals as Strings in arguments,
  rather than IDs.

## v0.8.0 - 2020-08-16

### Breaking Changes

- Integer fields are now i32 rather than i64 inline with the GraphQL spec. If
  larger integers are required a custom scalar should be used.
- The `cynic_arguments` attribute for passing arguments to GraphQL fields is
  now named `arguments`

### New Features

- querygen-web now incorporates graphiql & graphiql explorer, to make testing &
  building queries easier.
- querygen now supports bare selection sets, they're assumed to be queries.
  Quite easy to create these in GraphiQL/GraphqlExplorer, and they work for
  queries so seemed important.
- Arguments are now converted using the `IntoArgument<T>` trait - default
  conversions are provided for `Option` and reference types, so users don't
  always have to wrap Options in `Some` or explicitly clone their arguments.
- As a result of the above, cynic will no longer stop compiling when a schema
  changes a required argument to optional.

### Bug Fixes

- Fixed an issue with cynic-querygen where it guessed the name for the root of
  a query and crashed out if it was wrong (which was often).
- Fixed an issue where querygen would fail if given a query with a hardcoded
  enum value (#33)
- Integers are now i32 rather than i64, inline with the GraphQL spec. If
  larger integers are required a custom scalar should be used.
- Querygen now puts `argument_struct` attrs on types that have arguments rather
  than just types that have children with arguments. (#37)
- Fixed an issue where querygen would use the name of the query as the
  `graphql_type` on the root struct of named queries.
- Fixed a bunch of broken links in the book.

## v0.7.0 - 2020-06-23

### Breaking Changes

- `SerializableArgument`s are now required to be `Send`. Found this was
  required for using cynic in an async context. May revisit at some point to
  see if it's 100% required.

## v0.6.0 - 2020-06-17

### New Features

- `cynic::Id` now derives PartialEq, Hash & Eq
- Added `cynic::Id::new` function

### Bug Fixes

- Using a `query_module` should no longer cause errors on an individual derive
  to be attributed to the `query_module` span - the error information should
  now be associated with the derive it originated from.
- Fixed some dead code warnings in the selection builders output by query DSL

## v0.5.0 - 2020-06-14

### Breaking Changes

- `Query::body` no longer exists, the `Query` itself is now directly
  serializable, and exposes the `query` type itself. Errors that were
  previously exposed by `Query::body()` will now be surfaced when serializing a
  Query.
- `Argument::new` has been updated to take a `SerialiableArgument` itself.
- Removed `selection_set::Error`.
- `SerializableArgument::serialize` & `Scalar::encode` now return
  `Box<std::error::Error>` errors rather than `()`

### Bug Fixes

- `cynic-codegen` will now build with the rustfmt feature disabled.
- Removed some unwraps that I lazily put in and forgot to remove.

## v0.4.0 - 2020-06-12

### Breaking Changes

- `schema_path` parameters are now relative to `CARGO_MANFIEST_DIR` rather than
  the current working directory. This fixes an issue with cargo workspace
  projects where doc-tests would be relative to the sub-crate but cargo builds
  were relative to the workspace root. For projects not using workspaces this
  will probably make no difference
- The `query_dsl` has been reworked. Before each field with arguments had one
  or two structs: one for optional arguments & one for required arguments, and
  these were passed to the selector function before the selection set argument.
  Now each selector function takes the required arguments, and then returns a
  struct that follows the builder pattern to allow for optional arguments to be
  added. This fixes a few issues and is a bit more ergonomic.

### Bug Fixes

- Fixed a bug where any type used as an optional argument needed to implement
  Default. This was fixed by the `query_dsl` rework.
- Fixed a bug where optional arguments that were enums or interfaces had to be
  provided or type inference problems occurred. This was also fixed by the
  `query_dsl` rework.

## v0.3.0 - 2020-06-12

### New Features

- Added chrono::DateTime scalar support behind a chrono feature flag.
- The `cynic::selection_set` module and all it's contents are now documented.
- `QueryBody` now exposes it's arguments & query fields for greater flexibility
  (and use in snapshot testing etc.)

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
- `ID` fields are now correctly given the `cynic::Id` type in `query_dsl` - previously
  they were being forced to String.
- `cynic::Id` is now a `cynic::Scalar`
- Fixed an issue in `derive(QueryFragment)` where Enums inside lists would not be
  treated as Enums.
- `DecodeError` now implements `std::error::Error`

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
  define their own scalars. Also added support for this to cynic-querygen
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
