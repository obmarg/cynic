#![deny(rust_2018_idioms)]
//! # Cynic
//!
//! Cynic is a GraphQL query builder & data mapper for Rust.
//!
//! This documentation is primarily intended to be a reference for specific
//! functions, for a guide to using `Cynic` see the website: [cynic-rs.dev](https://cynic-rs.dev).
//!
//! ## Overview
//!
//! To get started with Cynic you'll need a GraphQL schema for the API you wish
//! to query.  The examples will be using the star wars API.
//!
//! ### Generating a Query DSL
//!
//! Once you've got your schema installed locally, you'll need to use the
//! `use_schema` macro to generate a schema module:
//!
//! ```rust
//! mod schema {
//!     cynic::use_schema!("../schemas/starwars.schema.graphql");
//! }
//! ```
//!
//! This macro generates a few things:
//!
//! 1. Some structs to represent the Input types the underlying schema.
//!    You may need to use these to build mutations or as parameters to queries.
//! 2. Definitions of all the Enums in the provided schema.  You'll need these
//!    if you want to use any enum types.
//! 3. Type safe selection set functions.  These can be used to build up a query
//!    manually, though it's usually easier to use the `QueryFragment` derive
//!    functionality explained below.  Hopefully you'll not need to use these
//!    directly too often.
//!
//! Though using macros to generate these is convenient, it does leave a lot of
//! code to the imagination.  You can get a glimpse of the things this defines
//! by running `cargo doc --document-private-items` and having a look in the
//! `schema` module. It's not ideal, but at least provides some visibility into
//! the various enum types.
//!
//! ### Creating QueryFragments
//!
//! Now that you have a schema defined, you can start building some queries.
//! Cynic lets you do this by deriving `QueryFragment` for a struct.  For
//! example, if we wanted to know what director title & director a Star Wars
//! film had, we could define this `QueryFragment`:
//!
//! ```rust
//! # mod schema {
//! #   cynic::use_schema!("../schemas/starwars.schema.graphql");
//! # }
//!
//! #[derive(cynic::QueryFragment)]
//! #[cynic(schema_path = "../schemas/starwars.schema.graphql")]
//! struct Film {
//!     title: Option<String>,
//!     director: Option<String>,
//! }
//!
//! // This `Film` struct can now be used as the type of a field on any other
//! // `QueryFragment` struct and cynic will know how to turn that into a GraphQL
//! // query, and populate the `Film` struct from the response.
//!
//! // For example, if we wanted to know the Director for a particular film:
//!
//! #[derive(cynic::QueryFragment)]
//! #[cynic(
//!     schema_path = "../schemas/starwars.schema.graphql",
//!     graphql_type = "Root"
//! )]
//! struct FilmDirectorQuery {
//!     // Here we use the `#[arguments()]` attribute on the `film` field to provide a
//!     // hard coded film ID to look up.  Though useful for demonstration, hard coded
//!     // arguments like this aren't much use in reality.  For more details on providing
//!     // runtime arguments please see below.
//!     #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
//!     film: Option<Film>,
//! }
//!
//! // You can then build a `cynic::Operation` from this fragment
//! use cynic::QueryBuilder;
//! let operation = FilmDirectorQuery::build(());
//! ```
//!
//! `operation` above implements `serde::Serialize` so can be used with any HTTP
//! client.  A selection of HTTP client integrations are provided in
//! `cynic::http` - see the docs there for examples of using a
//! `cynic::Operation`
//!
//! ```rust,ignore
//! let response = reqwest::blocking::Client::new()
//!                     .post("a_url")
//!                     .json(&operation)
//!                     .send()?;
//! let result = response.json::<GraphQlResponse<FilmDirectorQuery>>()?;
//! ```
//!
//! After this code has run, result will be an instance of
//! `GraphQlResponse<FilmDirectorQuery>` with the film populated appropriately.
//!
//! ### Dynamic Query Arguments
//!
//! The query above was useful for demonstration, but you'll usually want to be
//! able to provide parameters to your query.  To do this, you should define a
//! struct that contains all of the parameters you want to provide:
//!
//! ```rust
//! # mod schema {
//! #   cynic::use_schema!("../schemas/starwars.schema.graphql");
//! # }
//!
//! # #[derive(cynic::QueryFragment)]
//! # #[cynic(
//! #    schema_path = "../schemas/starwars.schema.graphql",
//! # )]
//! # struct Film {
//! #    title: Option<String>,
//! #    director: Option<String>
//! # }
//! // Deriving `QueryVariables` allows this struct to be used as variables in a
//! // `QueryFragment`, whether it represents part of a query or a whole query.
//! #[derive(cynic::QueryVariables)]
//! struct FilmArguments {
//!     id: Option<cynic::Id>,
//! }
//!
//! // You can now define a query to use these arguments on.  For example, to make
//! // `FilmDirectorQuery` a bit more dynamic:
//! #[derive(cynic::QueryFragment)]
//! #[cynic(
//!     schema_path = "../schemas/starwars.schema.graphql",
//!     graphql_type = "Root",
//!     // By adding the `variables` parameter to our `QueryFragment` we've made a variable
//!     // named `args` available for use in the `arguments` attribute.
//!     variables = "FilmArguments"
//! )]
//! struct FilmDirectorQueryWithArgs {
//!     // Here we use `args`, which we've declared above to be an instance of `FilmArguments`
//!     #[arguments(id: $id)]
//!     film: Option<Film>,
//! }
//!
//! // Then we can build a query using this new struct;
//! use cynic::QueryBuilder;
//! let operation = FilmDirectorQueryWithArgs::build(FilmArguments {
//!     id: Some("ZmlsbXM6MQ==".into()),
//! });
//! ```
//!
//! ## Feature Flags
//!
//! Cynic has a few features that are controlled by feature flags.
//!
//! - `http-surf` adds integration with the [`surf`](https://github.com/http-rs/surf)
//!   http client.
//! - `http-reqwest` adds async integration with the [`reqwest`](https://github.com/seanmonstar/reqwest)
//!   http client.
//! - `http-reqwest-blocking` adds blocking integration with the [`reqwest`](https://github.com/seanmonstar/reqwest)
//!   http client.
//! - `rkyv` can be used to speed up compiles when working with large schemas.
//!
//! It's worth noting that each of these features pulls in extra
//! dependencies, which may impact your build size.  Particularly
//! if you're targeting WASM.  In particular the `url` crate has
//! [known issues](https://github.com/servo/rust-url/issues/557) when
//! targeting web assembly.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

mod builders;
mod core;
mod id;
mod maybe_undefined;
mod operation;
mod result;

pub mod coercions;
pub mod queries;
pub mod variables;

pub mod http;
pub mod schema;

#[path = "private/mod.rs"]
pub mod __private;

pub use {
    self::core::{Enum, InlineFragments, InputObject, QueryFragment},
    builders::{MutationBuilder, QueryBuilder, SubscriptionBuilder},
    id::Id,
    maybe_undefined::MaybeUndefined,
    operation::{Operation, OperationBuildError, OperationBuilder, StreamingOperation},
    result::*,
    variables::{QueryVariableLiterals, QueryVariables, QueryVariablesFields},
};

pub use cynic_proc_macros::{
    schema, schema_for_derives, use_schema, Enum, InlineFragments, InputObject, QueryFragment,
    QueryVariableLiterals, QueryVariables, Scalar,
};

pub use static_assertions::assert_type_eq_all;

// We re-export serde as the output from a lot of our derive macros require it,
// and this way we can point at our copy rather than forcing users to add it to
// their Cargo.toml
pub use serde;

/// Implements a set of scalar traits for the given type & type lock.
///
/// For example, to use `uuid::Uuid` for a `Uuid` type defined in a schema:
///
/// ```rust
/// # #[macro_use] extern crate cynic;
/// # // Faking the uuid module here because it's easier than
/// # // actually pulling it in
/// #
/// # mod schema { cynic::use_schema!("../schemas/test_cases.graphql"); }
/// # mod uuid { pub struct Uuid(String); }
/// impl_scalar!(uuid::Uuid, schema::UUID);
/// ```
///
/// This macro can be used on any type that implements `serde::Serialize`,
/// provided the `schema` is defined in the current crate
#[macro_export]
macro_rules! impl_scalar {
    ($type:path, $type_lock:ident) => {
        $crate::impl_scalar!($type, $type_lock, self);
    };
    ($type:path, $type_lock_segment:ident $(:: $type_lock_rest :ident)::+) => {
        $crate::impl_scalar!($type, $($type_lock_rest)::*, $type_lock_segment);
    };
    ($type:path, $type_lock_segment:ident $(:: $type_lock_rest :ident)::+, $schema_module:ident $(:: $schema_module_rest : ident)*) => {
        $crate::impl_scalar!($type, $($type_lock_rest)::*, $schema_module(::$schema_module_rest)*::$type_lock_segment)
    };
    ($type:path, $type_lock:ident, $schema_module:ident $(:: $schema_module_rest : ident)*) => {
        #[automatically_derived]
        impl $crate::schema::IsScalar<$schema_module$(::$schema_module_rest)*::$type_lock> for $type {
            type SchemaType = $schema_module$(::$schema_module_rest)*::$type_lock;
        }

        // We derive a simple CoercesTo here, but since we don't own $type we can't
        // impl any of the more advanced coercions.
        #[automatically_derived]
        impl $crate::coercions::CoercesTo<$schema_module$(::$schema_module_rest)*::$type_lock> for $type {}

        #[automatically_derived]
        impl $schema_module$(::$schema_module_rest)*::variable::Variable for $type {
            const TYPE: cynic::variables::VariableType = cynic::variables::VariableType::Named(
                <$schema_module$(::$schema_module_rest)*::$type_lock as $crate::schema::NamedType>::NAME,
            );
        }
    };
}

#[macro_export(local_inner_macros)]
/// Asserts that the type implements _all_ of the given traits.
macro_rules! assert_impl {
    ($type:ty [$($impl_generics: tt)*] [$($where_clause: tt)*]: $($trait:path),+ $(,)?) => {
        const _: () = {
            // Only callable when `$type` implements all traits in `$($trait)+`.
            fn assert_impl_all<T: ?Sized $(+ $trait)+>() {}
            fn do_assert $($impl_generics)* () $($where_clause)* {
                assert_impl_all::<$type>();
            }
        };
    };
}
