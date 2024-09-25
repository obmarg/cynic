use std::str::FromStr;

use indexmap::IndexSet;
use iter::Iter;

pub mod ids;
pub mod iter;
pub mod writer;

mod definitions;
mod generated;
mod schemas;
mod string_literal;
mod types;
mod values;

use crate::common::IdRange;

pub use self::{
    definitions::{Definition, TypeDefinition},
    generated::{
        arguments::Argument,
        descriptions::Description,
        directives::{Directive, DirectiveDefinition},
        enums::{EnumDefinition, EnumValueDefinition},
        fields::FieldDefinition,
        input_objects::InputObjectDefinition,
        input_values::InputValueDefinition,
        interfaces::InterfaceDefinition,
        objects::ObjectDefinition,
        scalars::ScalarDefinition,
        schemas::{RootOperationTypeDefinition, SchemaDefinition},
        unions::{UnionDefinition, UnionMember},
    },
    string_literal::{StringLiteral, StringLiteralKind},
    types::Type,
    values::Value,
};
use self::{ids::*, storage::DefinitionRecord};

#[derive(Default)]
pub struct TypeSystemDocument {
    strings: IndexSet<Box<str>>,
    block_strings: Vec<Box<str>>,

    definitions: Vec<storage::DefinitionRecord>,

    schema_definitions: Vec<storage::SchemaDefinitionRecord>,
    scalar_definitions: Vec<storage::ScalarDefinitionRecord>,
    object_definitions: Vec<storage::ObjectDefinitionRecord>,
    interface_definitions: Vec<storage::InterfaceDefinitionRecord>,
    union_definitions: Vec<storage::UnionDefinitionRecord>,
    enum_definitions: Vec<storage::EnumDefinitionRecord>,
    input_object_definitions: Vec<storage::InputObjectDefinitionRecord>,
    directive_definitions: Vec<storage::DirectiveDefinitionRecord>,

    root_operation_definitions: Vec<storage::RootOperationTypeDefinitionRecord>,

    field_definitions: Vec<storage::FieldDefinitionRecord>,
    input_value_definitions: Vec<storage::InputValueDefinitionRecord>,
    enum_value_definitions: Vec<storage::EnumValueDefinitionRecord>,
    union_members: Vec<storage::UnionMemberRecord>,

    type_references: Vec<storage::TypeRecord>,

    values: Vec<storage::ValueRecord>,
    directives: Vec<storage::DirectiveRecord>,
    arguments: Vec<storage::ArgumentRecord>,
    descriptions: Vec<storage::DescriptionRecord>,
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

    pub(crate) fn all_locations() -> &'static [&'static str] {
        &[
            "QUERY",
            "MUTATION",
            "SUBSCRIPTION",
            "FIELD",
            "FRAGMENT_DEFINITION",
            "FRAGMENT_SPREAD",
            "INLINE_FRAGMENT",
            "SCHEMA",
            "SCALAR",
            "OBJECT",
            "FIELD_DEFINITION",
            "ARGUMENT_DEFINITION",
            "INTERFACE",
            "UNION",
            "ENUM",
            "ENUM_VALUE",
            "INPUT_OBJECT",
            "INPUT_FIELD_DEFINITION",
            "VARIABLE_DEFINITION",
        ]
    }
}

#[derive(Debug)]
pub struct MalformedDirectiveLocation(pub String);

impl FromStr for DirectiveLocation {
    type Err = MalformedDirectiveLocation;

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
            _ => return Err(MalformedDirectiveLocation(s.to_string())),
        })
    }
}

impl std::fmt::Display for DirectiveLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub trait TypeSystemId: Copy {
    type Reader<'a>: From<ReadContext<'a, Self>>;

    fn read(self, ast: &TypeSystemDocument) -> Self::Reader<'_> {
        ReadContext {
            id: self,
            document: ast,
        }
        .into()
    }
}

#[derive(Clone, Copy)]
pub struct ReadContext<'a, I> {
    id: I,
    document: &'a TypeSystemDocument,
}

impl super::TypeSystemDocument {
    pub fn read<T>(&self, id: T) -> T::Reader<'_>
    where
        T: TypeSystemId,
    {
        ReadContext { id, document: self }.into()
    }
}

impl TypeSystemDocument {
    pub fn definitions(&self) -> Iter<'_, Definition<'_>> {
        Iter::new(
            IdRange::new(
                DefinitionId::new(0),
                DefinitionId::new(self.definitions.len()),
            ),
            self,
        )
    }
}

pub mod storage {
    pub use super::{
        definitions::DefinitionRecord,
        generated::{
            arguments::ArgumentRecord,
            descriptions::DescriptionRecord,
            directives::{DirectiveDefinitionRecord, DirectiveRecord},
            enums::EnumDefinitionRecord,
            enums::EnumValueDefinitionRecord,
            fields::FieldDefinitionRecord,
            input_objects::InputObjectDefinitionRecord,
            input_values::InputValueDefinitionRecord,
            interfaces::InterfaceDefinitionRecord,
            objects::ObjectDefinitionRecord,
            scalars::ScalarDefinitionRecord,
            schemas::{RootOperationTypeDefinitionRecord, SchemaDefinitionRecord},
            unions::{UnionDefinitionRecord, UnionMemberRecord},
        },
        types::TypeRecord,
        values::ValueRecord,
    };
}
