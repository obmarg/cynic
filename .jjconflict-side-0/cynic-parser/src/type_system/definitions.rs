use crate::AstLookup;

use super::{
    ids::*,
    iter::{IdReader, Iter},
    DefinitionId, Directive, DirectiveDefinition, EnumDefinition, InputObjectDefinition,
    InterfaceDefinition, ObjectDefinition, ReadContext, ScalarDefinition, SchemaDefinition,
    TypeSystemId, UnionDefinition,
};

#[derive(Clone, Copy)]
pub enum DefinitionRecord {
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

#[derive(Clone, Copy)]
pub enum Definition<'a> {
    Schema(SchemaDefinition<'a>),
    SchemaExtension(SchemaDefinition<'a>),
    Type(TypeDefinition<'a>),
    TypeExtension(TypeDefinition<'a>),
    Directive(DirectiveDefinition<'a>),
}

impl Definition<'_> {
    pub fn span(&self) -> crate::Span {
        match self {
            Definition::Schema(def) | Definition::SchemaExtension(def) => def.span(),
            Definition::Type(ty) | Definition::TypeExtension(ty) => ty.span(),
            Definition::Directive(def) => def.span(),
        }
    }
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

impl<'a> TypeDefinition<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            TypeDefinition::Scalar(inner) => inner.name(),
            TypeDefinition::Object(inner) => inner.name(),
            TypeDefinition::Interface(inner) => inner.name(),
            TypeDefinition::Union(inner) => inner.name(),
            TypeDefinition::Enum(inner) => inner.name(),
            TypeDefinition::InputObject(inner) => inner.name(),
        }
    }

    pub fn directives(&self) -> Iter<'a, Directive<'a>> {
        match self {
            TypeDefinition::Scalar(inner) => inner.directives(),
            TypeDefinition::Object(inner) => inner.directives(),
            TypeDefinition::Interface(inner) => inner.directives(),
            TypeDefinition::Union(inner) => inner.directives(),
            TypeDefinition::Enum(inner) => inner.directives(),
            TypeDefinition::InputObject(inner) => inner.directives(),
        }
    }
}

impl TypeSystemId for DefinitionId {
    type Reader<'a> = Definition<'a>;
}

impl IdReader for Definition<'_> {
    type Id = DefinitionId;
}

impl<'a> From<ReadContext<'a, DefinitionId>> for Definition<'a> {
    fn from(value: ReadContext<'a, DefinitionId>) -> Self {
        let document = value.document;
        match document.lookup(value.id) {
            DefinitionRecord::Schema(id) => Definition::Schema(document.read(*id)),
            DefinitionRecord::Scalar(id) => {
                Definition::Type(TypeDefinition::Scalar(document.read(*id)))
            }
            DefinitionRecord::Object(id) => {
                Definition::Type(TypeDefinition::Object(document.read(*id)))
            }
            DefinitionRecord::Interface(id) => {
                Definition::Type(TypeDefinition::Interface(document.read(*id)))
            }
            DefinitionRecord::Union(id) => {
                Definition::Type(TypeDefinition::Union(document.read(*id)))
            }
            DefinitionRecord::Enum(id) => {
                Definition::Type(TypeDefinition::Enum(document.read(*id)))
            }
            DefinitionRecord::InputObject(id) => {
                Definition::Type(TypeDefinition::InputObject(document.read(*id)))
            }
            DefinitionRecord::SchemaExtension(id) => {
                Definition::SchemaExtension(document.read(*id))
            }
            DefinitionRecord::ScalarExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Scalar(document.read(*id)))
            }
            DefinitionRecord::ObjectExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Object(document.read(*id)))
            }
            DefinitionRecord::InterfaceExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Interface(document.read(*id)))
            }
            DefinitionRecord::UnionExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Union(document.read(*id)))
            }
            DefinitionRecord::EnumExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Enum(document.read(*id)))
            }
            DefinitionRecord::InputObjectExtension(id) => {
                Definition::TypeExtension(TypeDefinition::InputObject(document.read(*id)))
            }
            DefinitionRecord::Directive(id) => Definition::Directive(document.read(*id)),
        }
    }

    pub fn span(&self) -> crate::Span {
        match self {
            TypeDefinition::Scalar(inner) => inner.span(),
            TypeDefinition::Object(inner) => inner.span(),
            TypeDefinition::Interface(inner) => inner.span(),
            TypeDefinition::Union(inner) => inner.span(),
            TypeDefinition::Enum(inner) => inner.span(),
            TypeDefinition::InputObject(inner) => inner.span(),
        }
    }
}
