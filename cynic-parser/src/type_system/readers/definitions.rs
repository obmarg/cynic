use super::{
    directives::DirectiveDefinition, enums::EnumDefinition, input_objects::InputObjectDefinition,
    interfaces::InterfaceDefinition, objects::ObjectDefinition, scalars::ScalarDefinition,
    schemas::SchemaDefinition, unions::UnionDefinition, Directive,
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

    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        let rv: Box<dyn ExactSizeIterator<Item = Directive<'a>> + 'a> = match self {
            TypeDefinition::Scalar(inner) => Box::new(inner.directives()),
            TypeDefinition::Object(inner) => Box::new(inner.directives()),
            TypeDefinition::Interface(inner) => Box::new(inner.directives()),
            TypeDefinition::Union(inner) => Box::new(inner.directives()),
            TypeDefinition::Enum(inner) => Box::new(inner.directives()),
            TypeDefinition::InputObject(inner) => Box::new(inner.directives()),
        };

        rv
    }
}
