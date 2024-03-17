use std::str::FromStr;

use ids::*;
use indexmap::IndexSet;

pub mod ids;
pub mod readers;
pub mod storage;
pub mod writer;

// TODO: Possibly pub use all readers
use crate::span::Span;
pub use readers::{Definition, TypeDefinition, ValueReader};

use storage::*;

#[derive(Default)]
pub struct TypeSystemDocument {
    strings: IndexSet<Box<str>>,

    definitions: Vec<AstDefinition>,

    schema_definitions: Vec<SchemaDefinition>,
    scalar_definitions: Vec<ScalarDefinition>,
    object_definitions: Vec<ObjectDefinition>,
    interface_definitions: Vec<InterfaceDefinition>,
    union_definitions: Vec<UnionDefinition>,
    enum_definitions: Vec<EnumDefinition>,
    input_object_definitions: Vec<InputObjectDefinition>,
    directive_definitions: Vec<DirectiveDefinition>,

    field_definitions: Vec<FieldDefinition>,
    input_value_definitions: Vec<InputValueDefinition>,
    enum_value_definitions: Vec<EnumValueDefinition>,

    type_references: Vec<Type>,

    string_literals: Vec<StringLiteral>,

    values: Vec<Value>,
    directives: Vec<Directive>,
    arguments: Vec<Argument>,
}

#[derive(Clone, Copy)]
pub(crate) enum AstDefinition {
    Schema(SchemaDefinitionId),
    Scalar(ScalarDefinitionId),
    Object(ObjectDefinitionId),
    Interface(InterfaceDefinitionId),
    Union(UnionDefinitionId),
    Enum(EnumDefinitionId),
    InputObject(InputObjectDefinitionId),
    SchemaExtension(SchemaDefinitionId),
    ScalarExtension(ScalarDefinitionId),
    ObjectExtension(ObjectDefinitionId),
    InterfaceExtension(InterfaceDefinitionId),
    UnionExtension(UnionDefinitionId),
    EnumExtension(EnumDefinitionId),
    InputObjectExtension(InputObjectDefinitionId),
    Directive(DirectiveDefinitionId),
}

#[derive(Clone, Copy, Debug)]
pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
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
    VariableDefinition,
}

impl DirectiveLocation {
    pub fn as_str(self) -> &'static str {
        match self {
            DirectiveLocation::Query => "QUERY",
            DirectiveLocation::Mutation => "MUTATION",
            DirectiveLocation::Subscription => "SUBSCRIPTION",
            DirectiveLocation::Field => "FIELD",
            DirectiveLocation::FragmentDefinition => "FRAGMENT_DEFINITION",
            DirectiveLocation::FragmentSpread => "FRAGMENT_SPREAD",
            DirectiveLocation::InlineFragment => "INLINE_FRAGMENT",
            DirectiveLocation::Schema => "SCHEMA",
            DirectiveLocation::Scalar => "SCALAR",
            DirectiveLocation::Object => "OBJECT",
            DirectiveLocation::FieldDefinition => "FIELD_DEFINITION",
            DirectiveLocation::ArgumentDefinition => "ARGUMENT_DEFINITION",
            DirectiveLocation::Interface => "INTERFACE",
            DirectiveLocation::Union => "UNION",
            DirectiveLocation::Enum => "ENUM",
            DirectiveLocation::EnumValue => "ENUM_VALUE",
            DirectiveLocation::InputObject => "INPUT_OBJECT",
            DirectiveLocation::InputFieldDefinition => "INPUT_FIELD_DEFINITION",
            DirectiveLocation::VariableDefinition => "VARIABLE_DEFINITION",
        }
    }
}

impl FromStr for DirectiveLocation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "QUERY" => DirectiveLocation::Query,
            "MUTATION" => DirectiveLocation::Mutation,
            "SUBSCRIPTION" => DirectiveLocation::Subscription,
            "FIELD" => DirectiveLocation::Field,
            "FRAGMENT_DEFINITION" => DirectiveLocation::FragmentDefinition,
            "FRAGMENT_SPREAD" => DirectiveLocation::FragmentSpread,
            "INLINE_FRAGMENT" => DirectiveLocation::InlineFragment,
            "SCHEMA" => DirectiveLocation::Schema,
            "SCALAR" => DirectiveLocation::Scalar,
            "OBJECT" => DirectiveLocation::Object,
            "FIELD_DEFINITION" => DirectiveLocation::FieldDefinition,
            "ARGUMENT_DEFINITION" => DirectiveLocation::ArgumentDefinition,
            "INTERFACE" => DirectiveLocation::Interface,
            "UNION" => DirectiveLocation::Union,
            "ENUM" => DirectiveLocation::Enum,
            "ENUM_VALUE" => DirectiveLocation::EnumValue,
            "INPUT_OBJECT" => DirectiveLocation::InputObject,
            "INPUT_FIELD_DEFINITION" => DirectiveLocation::InputFieldDefinition,
            "VARIABLE_DEFINITION" => DirectiveLocation::VariableDefinition,
            _ => return Err(()),
        })
    }
}

impl std::fmt::Display for DirectiveLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
