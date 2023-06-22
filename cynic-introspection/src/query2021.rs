//! Defines the types & results for running an introspection query against a server
//! supporting the [2021 GraphQL Specification][1]
//!
//! [1]: https://spec.graphql.org/October2021/

use super::query::schema;

pub use super::query::{
    Directive, DirectiveLocation, EnumValue, Field, FieldType, InputValue, NamedType, TypeKind,
};

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
/// The results of a [2021 GraphQL Specification][1] introspection query
///
/// Can be used with cynic to run an introspection query.
///
/// [1]: https://spec.graphql.org/October2021/
pub struct IntrospectionQuery {
    #[cynic(rename = "__schema")]
    /// The schema returned from the query
    pub introspected_schema: IntrospectedSchema,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "__Schema")]
/// The schema returned from a query
pub struct IntrospectedSchema {
    /// The `query` root operation type
    pub query_type: NamedType,
    /// The `mutation` root operation type if any
    pub mutation_type: Option<NamedType>,
    /// The `subscription` root operation type if any
    pub subscription_type: Option<NamedType>,
    /// All the types available in the schema
    pub types: Vec<Type>,
    /// All the directives available in the schema
    pub directives: Vec<Directive>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "__Type")]
/// Details about a type in the schema
pub struct Type {
    /// The kind of type this `Type` is describing
    pub kind: TypeKind,
    /// The name of the `Type`
    ///
    /// This is an `Option` but the use of this struct means it should never
    /// be `None`.
    pub name: Option<String>,
    /// A description of the type
    pub description: Option<String>,
    /// The fields of the type, if it is an object or interface
    #[arguments(includeDeprecated: true)]
    pub fields: Option<Vec<Field>>,
    /// The input fields of the type, if it is an input object
    pub input_fields: Option<Vec<InputValue>>,
    /// Any interfaces this type implements, if it is an object or interface
    pub interfaces: Option<Vec<NamedType>>,
    /// The values this type can be, if it is an enum
    #[arguments(includeDeprecated: true)]
    pub enum_values: Option<Vec<EnumValue>>,
    /// A list of types that can be represented by this type if it is a union,
    /// or the set of types that implement this interface if it is an interface
    pub possible_types: Option<Vec<NamedType>>,
    /// A URL pointing to a specification for this scalar, if there is one
    #[cynic(rename = "specifiedByURL")]
    pub specified_by_url: Option<String>,
}
