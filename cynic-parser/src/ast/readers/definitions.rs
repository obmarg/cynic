// TODO: should these even be labelled as _readers_?
// Possibly not...

use super::{
    directives::DirectiveDefinition, enums::EnumDefinition, input_objects::InputObjectDefinition,
    interfaces::InterfaceDefinition, objects::ObjectDefinition, scalars::ScalarDefinition,
    schemas::SchemaDefinition, unions::UnionDefinition,
};

#[derive(Clone, Copy)]
pub enum Definition<'a> {
    Schema(SchemaDefinition<'a>),
    SchemaExtension(SchemaDefinition<'a>),
    Type(TypeDefinition<'a>),
    TypeExtension(TypeDefinition<'a>),
    Directive(DirectiveDefinition<'a>),
}

#[derive(Clone, Copy)]
pub enum TypeDefinition<'a> {
    Scalar(ScalarDefinition<'a>),
    Object(ObjectDefinition<'a>),
    Interface(InterfaceDefinition<'a>),
    Union(UnionDefinition<'a>),
    Enum(EnumDefinition<'a>),
    InputObject(InputObjectDefinition<'a>),
}
