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
//! use cynic::{QueryBuilder, http::ReqwestExt};
//! use cynic_introspection::IntrospectionQuery;
//! # #[tokio::main]
//! # async fn main() {
//! # let server = graphql_mocks::mocks::swapi::serve().await;
//! # let url = server.url();
//! # let url = url.as_ref();
//!
//! // We can run an introspection query and unwrap the data contained within
//! let introspection_data = reqwest::Client::new()
//!     .post(url)
//!     .run_graphql(IntrospectionQuery::build(()))
//!     .await
//!     .unwrap()
//!     .data
//!     .unwrap();
//!
//! // And then convert it into a schema for easier use.
//! let schema = introspection_data.into_schema().unwrap();
//!
//! assert_eq!(schema.query_type, "Root");
//! # }
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
//! The fields available for introspection differ between these two versions.  By default
//! we query only for fields supported in the June 2018 specification.  You can request
//! a different version of the query using [InstrospectionQuery::with_capabilities]:
//!
//! ```rust
//! use cynic::http::ReqwestBlockingExt;
//! use cynic_introspection::{IntrospectionQuery, SpecificationVersion};
//!
//! // We can run an introspection query and unwrap the data contained within
//! let introspection_data = reqwest::blocking::Client::new()
//!     .post("https://spacex-production.up.railway.app/")
//!     .run_graphql(
//!         IntrospectionQuery::with_capabilities(
//!             SpecificationVersion::October2021.capabilities()
//!         )
//!     )
//!     .unwrap()
//!     .data
//!     .unwrap();
//!
//! // And then convert it into a schema for easier use.
//! let schema = introspection_data.into_schema().unwrap();
//!
//! assert_eq!(schema.query_type, "Query");
//! ```
//!
//! ### Detecting Capabilities
//!
//! `cynic-introspection` also provides [CapabilitiesQuery], a query which can
//! determine the capabilites of a remote GraphQL server.  This can be paired with
//! `Introspection::with_capabilities`:
//!
//! ```rust
//! use cynic::{QueryBuilder, http::ReqwestExt};
//! use cynic_introspection::{CapabilitiesQuery, IntrospectionQuery};
//! # #[tokio::main]
//! # async fn main() {
//! # let server = graphql_mocks::mocks::swapi::serve().await;
//! # let url = server.url();
//! # let url = url.as_ref();
//!
//! // First we run a capabilites query to check what the server supports
//! let capabilities = reqwest::Client::new()
//!     .post(url)
//!     .run_graphql(CapabilitiesQuery::build(()))
//!     .await
//!     .unwrap()
//!     .data
//!     .unwrap()
//!     .capabilities();
//!
//! // Now we can safely run introspection, only querying for what the server supports.
//! let introspection_data = reqwest::Client::new()
//!     .post(url)
//!     .run_graphql(IntrospectionQuery::with_capabilities(capabilities))
//!     .await
//!     .unwrap()
//!     .data
//!     .unwrap();
//!
//! // And then convert it into a schema for easier use.
//! let schema = introspection_data.into_schema().unwrap();
//!
//! assert_eq!(schema.query_type, "Root");
//! # }
//! ```
//!
//! [1]: http://spec.graphql.org/October2021/#sec-Introspection
//! [2]: https://cynic-rs.dev
//! [3]: http://spec.graphql.org/June2018
//! [4]: http://spec.graphql.org/October2021

mod capabilities;
mod detection;
pub mod query;
mod query_builder;
mod schema;

pub use capabilities::CapabilitySet;
#[doc(inline)]
pub use detection::{CapabilitiesQuery, SpecificationVersion};
#[doc(inline)]
pub use query::{DirectiveLocation, IntrospectionQuery};

pub use schema::*;
