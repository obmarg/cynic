use crate::{ast, Ast};

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
mod types;
mod unions;
mod values;

pub use self::{
    arguments::*, definitions::*, directives::*, enums::*, fields::*, input_objects::*,
    input_values::*, interfaces::*, objects::*, scalars::*, schemas::*, types::*, unions::*,
    values::*,
};

// TODO: Make this sealed maybe?
// also think about the name, clashes with existing AstId a bit
pub trait AstId: Copy {
    type Reader<'a>: From<ReadContext<'a, Self>>;

    fn read(self, ast: &Ast) -> Self::Reader<'_> {
        ReadContext { id: self, ast }.into()
    }
}

pub struct ReadContext<'a, I> {
    id: I,
    ast: &'a Ast,
}

impl super::Ast {
    pub fn read<T>(&self, id: T) -> T::Reader<'_>
    where
        T: AstId,
    {
        ReadContext { id, ast: self }.into()
    }
}

impl Ast {
    pub fn definitions(&self) -> impl Iterator<Item = Definition<'_>> + '_ {
        self.definitions.iter().map(|definition| match definition {
            ast::AstDefinition::Schema(id) => Definition::Schema(self.read(*id)),
            ast::AstDefinition::Scalar(id) => {
                Definition::Type(TypeDefinition::Scalar(self.read(*id)))
            }
            ast::AstDefinition::Object(id) => {
                Definition::Type(TypeDefinition::Object(self.read(*id)))
            }
            ast::AstDefinition::Interface(id) => {
                Definition::Type(TypeDefinition::Interface(self.read(*id)))
            }
            ast::AstDefinition::Union(id) => {
                Definition::Type(TypeDefinition::Union(self.read(*id)))
            }
            ast::AstDefinition::Enum(id) => Definition::Type(TypeDefinition::Enum(self.read(*id))),
            ast::AstDefinition::InputObject(id) => {
                Definition::Type(TypeDefinition::InputObject(self.read(*id)))
            }
            ast::AstDefinition::SchemaExtension(id) => Definition::SchemaExtension(self.read(*id)),
            ast::AstDefinition::ScalarExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Scalar(self.read(*id)))
            }
            ast::AstDefinition::ObjectExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Object(self.read(*id)))
            }
            ast::AstDefinition::InterfaceExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Interface(self.read(*id)))
            }
            ast::AstDefinition::UnionExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Union(self.read(*id)))
            }
            ast::AstDefinition::EnumExtension(id) => {
                Definition::TypeExtension(TypeDefinition::Enum(self.read(*id)))
            }
            ast::AstDefinition::InputObjectExtension(id) => {
                Definition::TypeExtension(TypeDefinition::InputObject(self.read(*id)))
            }
            ast::AstDefinition::Directive(id) => Definition::Directive(self.read(*id)),
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
