#![deny(missing_docs)]
//! `cynic-introspection` defines a [GraphQL introspection query][1] that can be
//! run using [`cynic`][2], a rust GraphQL client.
//!
//! This can be used for any reason you'd want to introspect a GraphQL server -
//! including when you're using cynic as a library in your own project.
//!
//! It also provides a [Schema] abstraction on top of the introspection query
//! results, which provides some stronger typing than using the introspection
//! results directly.
//!
//! ```rust
//! use cynic::{QueryBuilder, http::ReqwestBlockingExt};
//! use cynic_introspection::IntrospectionQuery;
//!
//! // We can run an introspection query and unwrap the data contained within
//! let introspection_data = reqwest::blocking::Client::new()
//!     .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
//!     .run_graphql(IntrospectionQuery::build(()))
//!     .unwrap()
//!     .data
//!     .unwrap();
//!
//! // And then convert it into a schema for easier use.
//! let schema = introspection_data.into_schema().unwrap();
//!
//! assert_eq!(schema.query_type, "Root");
//! ```
//!
//! [1]: http://spec.graphql.org/October2021/#sec-Introspection
//! [2]: https://cynic-rs.dev

pub mod query;
mod schema;

#[doc(inline)]
pub use query::{DirectiveLocation, IntrospectionQuery};

pub use schema::*;
