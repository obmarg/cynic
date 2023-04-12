pub use queries::*;

#[cynic::schema_for_derives(file = r#"src/schema.graphql"#, module = "schema")]
mod queries {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query")]
    pub struct IntrospectionQuery {
        #[cynic(rename = "__schema")]
        pub introspected_schema: IntrospectedSchema,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "__Schema")]
    pub struct IntrospectedSchema {
        pub query_type: NamedType,
        pub mutation_type: Option<NamedType>,
        pub subscription_type: Option<NamedType>,
        pub types: Vec<Type>,
        pub directives: Vec<Directive>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "__Directive")]
    pub struct Directive {
        pub name: String,
        pub description: Option<String>,
        pub args: Vec<InputValue>,
        pub locations: Vec<DirectiveLocation>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "__Type")]
    pub struct Type {
        pub kind: TypeKind,
        pub name: Option<String>,
        pub description: Option<String>,
        #[arguments(includeDeprecated: true)]
        pub fields: Option<Vec<Field>>,
        pub input_fields: Option<Vec<InputValue>>,
        pub interfaces: Option<Vec<NamedType>>,
        #[arguments(includeDeprecated: true)]
        pub enum_values: Option<Vec<EnumValue>>,
        pub possible_types: Option<Vec<NamedType>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "__EnumValue")]
    pub struct EnumValue {
        pub name: String,
        pub description: Option<String>,
        pub is_deprecated: bool,
        pub deprecation_reason: Option<String>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "__Field")]
    pub struct Field {
        pub name: String,
        pub description: Option<String>,
        pub args: Vec<InputValue>,
        #[cynic(rename = "type")]
        pub ty: FieldType,
        pub is_deprecated: bool,
        pub deprecation_reason: Option<String>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "__InputValue")]

    pub struct InputValue {
        pub name: String,
        pub description: Option<String>,
        #[cynic(rename = "type")]
        pub ty: FieldType,
        pub default_value: Option<String>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "__Type")]
    pub struct FieldType {
        pub kind: TypeKind,
        pub name: Option<String>,
        #[cynic(recurse = 5)]
        pub of_type: Option<Box<FieldType>>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "__Type")]
    pub struct NamedType {
        pub name: Option<String>,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug, PartialEq, Eq)]
    #[cynic(graphql_type = "__DirectiveLocation")]
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
    pub enum TypeKind {
        Scalar,
        Object,
        Interface,
        Union,
        Enum,
        InputObject,
        List,
        NonNull,
    }
}

mod schema {
    cynic::use_schema!(r#"src/schema.graphql"#);
}
