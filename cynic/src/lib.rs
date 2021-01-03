//! # Cynic
//!
//! Cynic is a GraphQL query builder & data mapper for Rust.
//!
//! This documentation is primarily intended to be a reference for specific functions,
//! for a guide to using `Cynic` see the website: [cynic-rs.dev](https://cynic-rs.dev).
//!
//! ## Overview
//!
//! To get started with Cynic you'll need a GraphQL schema for the API you wish to
//! query.  The examples will be using the star wars API.
//!
//! ### Generating a Query DSL
//!
//! Once you've got your schema installed locally, you'll need to use the
//! `query_dsl` macro to generate a query_dsl for your schema:
//!
//! ```rust
//! mod query_dsl {
//!     cynic::query_dsl!("../schemas/starwars.schema.graphql");
//! }
//! ```
//!
//! This macro generates a few things:
//!
//! 1. Some structs to represent the Input types the underlying schema.
//!    You may need to use these to build mutations or as parameters to queries.
//! 2. Definitons of all the Enums in the provided schema.  You'll need these if
//!    you want to use any enum types.
//! 3. Type safe selection set functions.  These can be used to build up a query
//!    manually, though it's usually easier to use the `QueryFragment` derive
//!    functionality explained below.  Hopefully you'll not need to use these
//!    directly too often.
//!
//! Though using macros to generate these is convenient, it does leave a lot of code
//! to the imagination.  You can get a glimpse of the things this defines by running
//! `cargo doc --document-private-items` and having a look in the `query_dsl` module.
//! It's not ideal, but at least provides some visibility into the various enum types.
//!
//! ### Creating QueryFragments
//!
//! Now that you have a query_dsl defined, you can start building some queries.
//! Cynic lets you do this by deriving `QueryFragment` for a struct.  For example,
//! if we wanted to know what director title & director a Star Wars film had, we
//! could define this `QueryFragment`:
//!
//! ```rust
//! # mod query_dsl {
//! #   cynic::query_dsl!("../schemas/starwars.schema.graphql");
//! # }
//!
//! #[derive(cynic::QueryFragment)]
//! #[cynic(
//!     schema_path = "../schemas/starwars.schema.graphql",
//!     query_module = "query_dsl",
//!     graphql_type = "Film"
//! )]
//! struct Film {
//!     title: Option<String>,
//!     director: Option<String>
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
//!     query_module = "query_dsl",
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
//! use cynic::{QueryBuilder};
//! let operation = FilmDirectorQuery::build(());
//!
//! ```
//!
//! `operation` above implements `serde::Serialize` so can be used with any HTTP
//! client.  A selection of HTTP client integrations are provided in
//! `cynic::http` - see the docs there for examples of using a `cynic::Operation`
//!
//! ```rust,ignore
//! let response = reqwest::blocking::Client::new()
//!                     .post("a_url")
//!                     .json(&operation)
//!                     .send()?;
//! let result = query.decode_response(response.json()?)?;
//! ```
//!
//! After this code has run, result will be an instance of `FilmDirectorQuery`
//! with the film populated appropriately.
//!
//! ### Dynamic Query Arguments
//!
//! The query above was useful for demonstration, but you'll usually want to be able to
//! provide parameters to your query.  To do this, you should define a struct that contains
//! all of the parameters you want to provide:
//!
//! ```rust
//! # mod query_dsl {
//! #   cynic::query_dsl!("../schemas/starwars.schema.graphql");
//! # }
//!
//! # #[derive(cynic::QueryFragment)]
//! # #[cynic(
//! #     schema_path = "../schemas/starwars.schema.graphql",
//! #    query_module = "query_dsl",
//! #    graphql_type = "Film"
//! # )]
//! # struct Film {
//! #    title: Option<String>,
//! #    director: Option<String>
//! # }
//! // Deriving `FragmentArguments` allows this struct to be used as arguments to a
//! // `QueryFragment` fragment, whether it represents part of a query or a whole query.
//! #[derive(cynic::FragmentArguments)]
//! struct FilmArguments {
//!     id: Option<cynic::Id>
//! }
//!
//! // You can now define a query to use these arguments on.  For example, to make
//! // `FilmDirectorQuery` a bit more dynamic:
//! #[derive(cynic::QueryFragment)]
//! #[cynic(
//!     schema_path = "../schemas/starwars.schema.graphql",
//!     query_module = "query_dsl",
//!     graphql_type = "Root",
//!     // By adding the `argument_struct` parameter to our `QueryFragment` we've made a variable
//!     // named `args` avaiable for use in the `arguments` attribute.
//!     argument_struct = "FilmArguments"
//! )]
//! struct FilmDirectorQueryWithArgs {
//!     // Here we use `args`, which we've declared above to be an instance of `FilmArguments`
//!     #[arguments(id = &args.id)]
//!     film: Option<Film>,
//! }
//!
//! // Then we can build a query using this new struct;
//! use cynic::QueryBuilder;
//! let operation = FilmDirectorQueryWithArgs::build(
//!     FilmArguments{ id: Some("ZmlsbXM6MQ==".into()) }
//! );
//! ```
//!
//! ## Feature Flags
//!
//! Cynic has a few features that are controlled by feature flags.
//!
//! - `chrono` adds support for chrono::DateTime scalars.
//! - `uuid` adds support for Uuid scalars
//! - `bson` adds support for ObjectId scalars
//! - `url` adds support for Url scalars
//! - `surf` adds integration with the [`surf`](https://github.com/http-rs/surf)
//!   http client.
//! - `transport-ws` provides struct definitions for implementing the
//!   graphql-transport-ws websocket protocol.
//! - `transport-ws-async-tungstenite` provides an implementation of
//!   graphql-transport-ws on top of `async-tungstenite`
//!
//! It's worth noting that each of these features pulls in extra
//! dependencies, which may impact your build size.  Particularly
//! if you're targetting WASM.  In particular the `url` crate has
//! [known issues](https://github.com/servo/rust-url/issues/557) when
//! targetting web assembly.

mod arguments;
mod builders;
mod fragments;
mod id;
mod integrations;
mod operation;
mod result;
mod scalar;

pub mod http;
pub mod protocol;
pub mod selection_set;
pub mod utils;

pub use json_decode::DecodeError;

pub use arguments::{Argument, FromArguments, IntoArgument, SerializableArgument};
pub use builders::{MutationBuilder, QueryBuilder, SubscriptionBuilder};
pub use fragments::{FragmentArguments, FragmentContext, InlineFragments, QueryFragment};
pub use id::Id;
pub use operation::{Operation, StreamingOperation};
pub use result::{GraphQLError, GraphQLResponse, GraphQLResult, PossiblyParsedData};
pub use scalar::Scalar;
pub use selection_set::SelectionSet;

pub use cynic_proc_macros::{
    query_dsl, query_module, Enum, FragmentArguments, InlineFragments, InputObject, QueryFragment,
    Scalar,
};

// We re-export serde_json as the output from a lot of our derive macros require it,
// and this way we can point at our copy rather than forcing users to add it to
// their Cargo.toml
pub use serde_json;

pub type SerializeError = Box<dyn std::error::Error + Send + Sync>;

/// A trait for GraphQL enums.
///
/// This trait is generic over some TypeLock which is used to tie an Enum
/// definition back into it's GraphQL enum.  Generally this will be some
/// type generated in the GQL code.
pub trait Enum<TypeLock>: Sized {
    fn select() -> SelectionSet<'static, Self, ()>;
}

/// A trait for GraphQL input objects.
///
/// This trait is generic over some TypeLock which is used to tie an InputType
/// back into it's GraphQL input object.  Generally this will be some type
/// generated in the GQL code.
///
/// It's recommended to derive this trait with the [InputObject](derive.InputObject.html)
/// derive.  You can also implement it yourself, but you'll be responsible
/// for implementing the `SerializableArgument` trait if you want to use it.
pub trait InputObject<TypeLock> {}

impl<TypeLock, T> InputObject<TypeLock> for &T where T: InputObject<TypeLock> {}
impl<TypeLock, T> InputObject<TypeLock> for Box<T> where T: InputObject<TypeLock> {}

/// A marker trait that indicates a particular type is at the root of a GraphQL schemas query
/// hierarchy.
pub trait QueryRoot {}

/// A marker trait that indicates a particular type is at the root of a GraphQL schemas
/// mutation hierarchy.
pub trait MutationRoot {}

/// A marker trait that indicates a particular type is at the root of a GraphQL schemas
/// subscription hierarchy.
pub trait SubscriptionRoot {}
