//! # Cynic
//!
//! Cynic is a GraphQL query builder & data mapper for Rust.  
//!
//! See [the README on GitHub](https://github.com/polyandglot/cynic) for more details.
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
//!     cynic::query_dsl!("swapi.graphql");
//! }
//! ```
//!
//! This macro will read the provided schema and generate some functions & structs
//! that can be used for querying it.  You shouldn't need to use a lot of the
//! functions - they mostly exist to encode the type rules of the GraphQL schmea
//! into Rust.  Higher level macros will usually make use of these for us, though if you
//! have a particularly unusual edge case you may need to dig into them.  
//! The structs & enums generated as part of this module may well be of use though.
//!
//! ### Creating QueryFragments
//!
//! Now that you have a query_dsl defined, you can start building some queries.
//! Cynic lets you do this by deriving "QueryFragment" for a struct.  For example,
//! if we wanted to know what director a Star Wars film had, we could use this
//! `QueryFragment`:
//!
//! ```rust
//! #[derive(cynic::QueryFragment)]
//! #[cynic(
//!     schema_path = "swapi.graphql",
//!     query_module = "query_dsl",
//!     graphql_type = "Film"
//! )]
//! struct Film {
//!     title: String,
//!     director: String
//! }
//! ```
//!
//! This `Film` struct can now be used as the type of a field on any other
//! `QueryFragment` struct and cynic will know what to do.
//!
//! For example, if we wanted to know the Director for a particular film:
//!
//! ```rust
//! #[derive(cynic::QueryFragment)]
//! #[cynic(
//!     schema_path = "swapi.graphql",
//!     query_module = "query_dsl",
//!     graphql_type = "Root"
//! )]
//! struct FilmDirectorQuery {
//!     #[cynic_arguments(id = "ZmlsbXM6MQ==")]
//!     film: Film,
//! }
//! ```
//!
//! Here we use the `#[cynic_arguments()]` attribute on the `film` field to provide a
//! hard coded film ID to look up.  Though useful for demonstration, hard coded
//! arguments like this aren't much use in reality.  For more details on providing
//! runtime arguments please see below.
//!
//! ### Sending Queries
//!
//! Notice that `FilmDirectorQuery` above defines it's `graphql_type` as `Root` - the root
//! query type in SWAPI.  Whenever you define a type at this level of the heirarchy it can
//! be used as a query on its own, rather than as part of another query.
//!
//! To send the FilmDirectorQuery above:
//!
//! ```rust
//! let query = cynic::Query::new(FilmDirectorQuery::fragment(()))
//! let response = reqwest::Client::new()
//!                     .post("a_url")
//!                     .json(query.body().unwrap())
//!                     .send();
//! let result = query.decode_response(response.json().await?).unwrap();
//! ```
//!
//! After this code has run, result will be an instance of FilmDirectorQuery
//! with the film populated appropriately.
//!
//! ### Dynamic Query Arguments
//!
//! The query above was useful for demonstration, but you'd usually want to be able to
//! provide parameters to your query.  To do this, you should define a struct that contains
//! all of the parameters you want to provide:
//!
//! ```rust
//! #[derive(cynic::FragmentArguments)]
//! struct FilmArguments {
//!     id: Option<String>
//! }
//! ```
//!
//! Deriving FragmentArguments allows this struct to be used as arguments on an
//! entire query or just part of a query.
//!
//! You can now define a query to use these arguments on.  For example, to make
//! FilmDirectorQuery a bit more dynamic:
//!
//! ```rust
//! #[derive(cynic::QueryFragment)]
//! #[cynic(
//!     schema_path = "swapi.graphql",
//!     query_module = "query_dsl",
//!     graphql_type = "Root",
//!     argument_struct = "FilmArguments"
//! )]
//! struct FilmDirectorQueryWithArgs {
//!     #[cynic_arguments(id = args.id)]
//!     film: Film,
//! }
//! ```
//!
//! By adding the `argument_struct` parameter to our QueryFragment we've made a variable
//! named `args` avaiable for use in the `cynic_arguments` attribute.  This `args` will
//! be an instance of `FilmArguments`, and will need to be provided whenever this is used
//! as a query.
//!
//! To build a query using this new struct:
//!
//! ```rust
//! let query = cynic::Query::new(
//!     FilmDirectorQueryWithArgs::fragment(
//!         FilmArguments{ id: "ZmlsbXM6MQ==".to_string() }
//!     )
//! );
//! ```
//!
use std::collections::HashMap;

mod argument;
mod field;
mod query;
mod result;
mod scalar;
pub mod selection_set;

pub use argument::Argument;
pub use query::Query;
pub use result::{GraphQLError, GraphQLResponse, GraphQLResult, PossiblyParsedData};
pub use scalar::Scalar;
pub use selection_set::SelectionSet;

pub trait QueryFragment {
    type SelectionSet;
    type Arguments: FragmentArguments;

    fn fragment(arguments: Self::Arguments) -> Self::SelectionSet;
}

/// A marker trait for the arguments types on QueryFragments.
///
/// We use this in combination with the IntoArguments trait below
/// to convert between different argument types in a query heirarchy.
pub trait FragmentArguments {}

impl FragmentArguments for () {}

/// Used for converting between different argument types in a QueryFragment
/// heirarchy.
///
/// For example if an outer QueryFragment has a struct with several parameters
/// but an inner QueryFragment needs none then we can use () as the arguments
/// type on the inner fragments and use the blanket implementation of IntoArguments
/// to convert to ().
///
/// Similarly, the
pub trait IntoArguments<T> {
    fn into_args(&self) -> T;
}

impl IntoArguments<()> for dyn FragmentArguments {
    fn into_args(&self) -> () {
        ()
    }
}

impl<T> IntoArguments<T> for T
where
    T: Clone,
{
    fn into_args(&self) -> T {
        // TODO: Figure out if there's a way to avoid this clone...
        self.clone()
    }
}

pub trait QueryRoot {}

#[derive(Debug, serde::Serialize)]
pub struct QueryBody<'a> {
    query: String,
    variables: HashMap<String, &'a serde_json::Value>,
}

pub use cynic_codegen::{query_dsl, FragmentArguments, QueryFragment};
