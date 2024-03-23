use crate::type_system::{self, TypeSystemDocument};

mod arguments;
mod definitions;
mod directives;
mod enums;
mod fields;
mod input_objects;
mod input_values;
mod interfaces;
mod objects;
mod scalars;
mod schemas;
mod string_literal;
mod types;
mod unions;
mod values;

pub use self::{
    arguments::*, definitions::*, directives::*, enums::*, fields::*, input_objects::*,
    input_values::*, interfaces::*, objects::*, scalars::*, schemas::*, string_literal::*,
    types::*, unions::*, values::*,
};

// TODO: Make this sealed maybe?
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
    pub fn definitions(&self) -> impl Iterator<Item = Definition<'_>> + '_ {
        self.definitions.iter().map(|definition| match definition {
            type_system::AstDefinition::Schema(id) => Definition::Schema(self.read(*id)),
            type_system::AstDefinition::Scalar(id) => {
                Definition::Type(TypeDefinition::Scalar(self.read(*id)))
            }
            type_system::AstDefinition::Object(id) => {
                Definition::Type(TypeDefinition::Object(self.read(*id)))
            }
            type_system::AstDefinition::Interface(id) => {
                Definition::Type(TypeDefinition::Interface(self.read(*id)))
            }
            type_system::AstDefinition::Union(id) => {
                Definition::Type(TypeDefinition::Union(self.read(*id)))
            }
            type_system::AstDefinition::Enum(id) => {
                Definition::Type(TypeDefinition::Enum(self.read(*id)))
            }
            type_system::AstDefinition::InputObject(id) => {
                Definition::Type(TypeDefinition::InputObject(self.read(*id)))
            }
            type_system::AstDefinition::SchemaExtension(id) => {
                Definition::SchemaExtension(self.read(*id))
            }
            type_system::AstDefinition::ScalarExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Scalar(self.read(*id)))
            }
            type_system::AstDefinition::ObjectExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Object(self.read(*id)))
            }
            type_system::AstDefinition::InterfaceExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Interface(self.read(*id)))
            }
            type_system::AstDefinition::UnionExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Union(self.read(*id)))
            }
            type_system::AstDefinition::EnumExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Enum(self.read(*id)))
            }
            type_system::AstDefinition::InputObjectExtension(id) => {
                Definition::TypeExtension(TypeDefinition::InputObject(self.read(*id)))
            }
            type_system::AstDefinition::Directive(id) => Definition::Directive(self.read(*id)),
        })
    }
}

// pub enum Definition<'a> {
//     Schema(AstReader<'a, SchemaDefinitionId>),
//     SchemaExtension(AstReader<'a, SchemaDefinitionId>),
//     Type(TypeDefinition<'a>),
//     TypeExtension(TypeDefinition<'a>),
//     Directive(AstReader<'a, DirectiveDefinitionId>),
// }

// pub enum TypeDefinition<'a> {
//     Scalar(AstReader<'a, ScalarDefinitionId>),
//     Object(AstReader<'a, ObjectDefinitionId>),
//     Interface(AstReader<'a, InterfaceDefinitionId>),
//     Union(AstReader<'a, UnionDefinitionId>),
//     Enum(AstReader<'a, EnumDefinitionId>),
//     InputObject(AstReader<'a, InputObjectDefinitionId>),
// }
