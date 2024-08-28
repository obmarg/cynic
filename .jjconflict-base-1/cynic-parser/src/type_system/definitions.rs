use super::{
    Directive, DirectiveDefinition, EnumDefinition, InputObjectDefinition, InterfaceDefinition,
    ObjectDefinition, ScalarDefinition, SchemaDefinition, UnionDefinition,
};

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
