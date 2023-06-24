//! Defines the types & results for running an introspection query against a server
//! supporting the [2018 GraphQL Specification][1]
//!
//! [1]: https://spec.graphql.org/June2018/

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
/// The results of a [2018 GraphQL Specification][1] introspection query
///
/// Can be used with cynic to run an introspection query.
///
/// [1]: https://spec.graphql.org/June2018/
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
#[cynic(graphql_type = "__Directive")]
/// A directive either used in the schema or available to queries
pub struct Directive {
    /// The name of the directive
    pub name: String,
    /// A description of the directive
    pub description: Option<String>,
    /// Any arguments that can be provided to the directive
    pub args: Vec<InputValue>,
    /// The locations where the directive may be used
    pub locations: Vec<DirectiveLocation>,
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
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "__EnumValue")]
/// Represents one of the possible values of an enum type
pub struct EnumValue {
    /// The name of the value
    pub name: String,
    /// A description of the value
    pub description: Option<String>,
    /// Whether the value is deprecated and should no longer be used.
    pub is_deprecated: bool,
    /// Optionally provides a reason why this enum value is deprecated
    pub deprecation_reason: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "__Field")]
/// Represents one of the fields of an object or interface type
pub struct Field {
    /// The name of the field
    pub name: String,
    /// A description of the field
    pub description: Option<String>,
    /// A list of arguments this field accepts.
    pub args: Vec<InputValue>,
    /// The type of value returned by this field
    #[cynic(rename = "type")]
    pub ty: FieldType,
    /// Whether this field is deprecated and should no longer be used.
    pub is_deprecated: bool,
    /// Optionally provides a reason why this field is deprecated
    pub deprecation_reason: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "__InputValue")]
/// Represents field and directive arguments as well as the fields of an input object.
pub struct InputValue {
    /// The name of the argument/field
    pub name: String,
    /// A description of the argument/field
    pub description: Option<String>,
    #[cynic(rename = "type")]
    /// The type of this argument/field
    pub ty: FieldType,
    /// An optional default value for this field, represented as a GraphQL literal
    pub default_value: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "__Type")]
/// The type of a [`Field`].
///
/// This may be either a wrapper or a named type, depending on the field in question
pub struct FieldType {
    /// The kind of type this `Type` is describing
    pub kind: TypeKind,
    /// The name of the `Type`
    ///
    /// This is an `Option` but the use of this struct means it should never
    /// be `None`.
    pub name: Option<String>,
    /// If `kind` is [TypeKind::List] or [TypeKind::NonNull] this contains the type
    /// that is wrapped.
    #[cynic(recurse = 6)]
    pub of_type: Option<Box<FieldType>>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "__Type")]
/// A named type
pub struct NamedType {
    /// The name of the named type.  This shouldn't be null
    pub name: Option<String>,
}

#[derive(cynic::Enum, Clone, Copy, Debug, PartialEq, Eq)]
#[cynic(graphql_type = "__DirectiveLocation")]
#[allow(missing_docs)]
/// A location where a directive can be used
pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(graphql_type = "__TypeKind")]
/// The "kind" of a type
pub enum TypeKind {
    /// Represents scalar types such as Int, String, and Boolean.
    Scalar,
    /// Object types represent concrete instantiations of sets of fields.
    Object,
    /// Interfaces are an abstract type where there are common fields declared.
    Interface,
    /// Unions are an abstract type where no common fields are declared.
    Union,
    /// Enums are special scalars that can only have a defined set of values.
    Enum,
    /// Input objects are composite types defined as a list of named input values.
    InputObject,
    /// Lists represent sequences of values in GraphQL. A List type is a type modifier:
    /// it wraps another type instance in the of_type field, which defines the type of
    /// each item in the list.
    List,
    /// GraphQL types are nullable by default.  A Non-Null type is a type modifier: it
    /// wraps another type instance in the ofType field. Non-null types do not allow
    /// null as a response, and indicate required inputs for arguments and input object
    /// fields.
    NonNull,
}

#[cynic::schema("introspection")]
pub(super) mod schema {}
