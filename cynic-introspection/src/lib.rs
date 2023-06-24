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
//! ### GraphQL Versions
//!
//! GraphQL servers currently commonly support two different versions of the GraphQL
//! specification:
//!
//! - [The June 2018 Specification][3]
//! - [The October 2021 Specification][4]
//!
//! The fields available for introspection differ between these two versions, so cynic
//! provides two different introspection queries in the modules [query2018] and [query2021]
//! respectively.
//!
//! It also provides [CapabilityDetectionQuery], a query which can determine which version
//! of the specification a GraphQL server supports.
//!
//! [1]: http://spec.graphql.org/October2021/#sec-Introspection
//! [2]: https://cynic-rs.dev
//! [3]: http://spec.graphql.org/June2018
//! [4]: http://spec.graphql.org/October2021

mod detection;
pub mod query2018;
pub mod query2021;
mod schema;

#[doc(inline)]
pub use detection::{CapabilityDetectionQuery, SpecificationVersion};
pub use query2018 as query;
#[doc(inline)]
pub use query2018::{DirectiveLocation, IntrospectionQuery};

pub use schema::*;
