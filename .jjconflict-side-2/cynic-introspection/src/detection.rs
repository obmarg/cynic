//! Defines types for an introspection query that can tell what a GraphQL server
//! supports.

use crate::capabilities::CapabilitySet;

use super::query::schema;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
/// A query that detects what capabilities a remote GraphQL server has, which can be used
/// to determine what introspection query should be made against that server.
///
/// Currently this just determines which version of the specification the server supports,
/// but it may be extended in the future to detect other capabilities e.g. which RFCs a
/// server has implemented support for.
///
/// ```rust
/// use cynic::{QueryBuilder, http::ReqwestBlockingExt};
/// use cynic_introspection::CapabilitiesQuery;
///
/// let data = reqwest::blocking::Client::new()
///     .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
///     .run_graphql(CapabilitiesQuery::build(()))
///     .unwrap()
///     .data
///     .unwrap();
///
/// let capabilities = data.capabilities();
/// println!("This server supports {capabilities:?}");
/// ```
pub struct CapabilitiesQuery {
    #[cynic(rename = "__type")]
    #[arguments(name: "__Type")]
    type_type: Option<Type>,
}

impl CapabilitiesQuery {
    /// The capabilities that were detected by this query
    pub fn capabilities(&self) -> CapabilitySet {
        CapabilitySet {
            specification_version: self.version_supported(),
        }
    }

    fn version_supported(&self) -> SpecificationVersion {
        let Some(type_type) = &self.type_type else {
            return SpecificationVersion::Unknown;
        };

        let specified_by_field = type_type
            .fields
            .iter()
            .find(|field| field.name == "specifiedByURL");

        match specified_by_field {
            Some(_) => SpecificationVersion::October2021,
            None => SpecificationVersion::June2018,
        }
    }
}

/// Versions of the GraphQL specification that the CapabilitiesQuery can detect.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SpecificationVersion {
    /// We were unable to determine which version of the GraphQL specification
    /// this server supports.
    Unknown,
    /// The GraphQL specification published in June 2018
    #[default]
    June2018,
    /// The GraphQL specification published in October 2021
    October2021,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "__Type")]
/// Details about a type in the schema
struct Type {
    #[cynic(flatten)]
    #[arguments(includeDeprecated: true)]
    fields: Vec<Field>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "__Field")]
/// Represents one of the fields of an object or interface type
struct Field {
    /// The name of the field
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordering_of_specification_version() {
        assert!(SpecificationVersion::June2018 < SpecificationVersion::October2021);
    }
}
