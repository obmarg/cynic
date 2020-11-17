mod fields;
mod parser;
mod type_index;
mod type_refs;

pub use fields::*;
pub use parser::*;
pub use type_index::{GraphPath, TypeIndex};
pub use type_refs::{InputTypeRef, OutputTypeRef};

use std::{convert::TryFrom, rc::Rc};

use crate::Error;

// TODO: Put these types somewhere sensible

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type<'schema> {
    Scalar(ScalarDetails<'schema>),
    Object(ObjectDetails<'schema>),
    Interface(InterfaceDetails<'schema>),
    Enum(EnumDetails<'schema>),
    Union(UnionDetails<'schema>),
    InputObject(InputObjectDetails<'schema>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScalarDetails<'schema> {
    pub name: &'schema str,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectDetails<'schema> {
    name: &'schema str,
    fields: Vec<OutputField<'schema>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InterfaceDetails<'schema> {
    name: &'schema str,
    fields: Vec<OutputField<'schema>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EnumDetails<'schema> {
    pub name: &'schema str,
    pub values: Vec<&'schema str>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnionDetails<'schema> {
    name: &'schema str,
    types: Vec<OutputTypeRef<'schema>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct InputObjectDetails<'schema> {
    pub name: &'schema str,
    pub fields: Vec<InputField<'schema>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputType<'schema> {
    Scalar(ScalarDetails<'schema>),
    Enum(EnumDetails<'schema>),
    InputObject(InputObjectDetails<'schema>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputType<'schema> {
    Scalar(ScalarDetails<'schema>),
    Object(ObjectDetails<'schema>),
    Interface(InterfaceDetails<'schema>),
    Enum(EnumDetails<'schema>),
    Union(UnionDetails<'schema>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LeafType<'schema> {
    Scalar(ScalarDetails<'schema>),
    Enum(EnumDetails<'schema>),
}

impl<'schema> Type<'schema> {
    fn from_type_defintion(
        type_def: &TypeDefinition<'schema>,
        type_index: &Rc<TypeIndex<'schema>>,
    ) -> Type<'schema> {
        match type_def {
            TypeDefinition::Scalar(scalar) => Type::Scalar(ScalarDetails { name: scalar.name }),
            TypeDefinition::Object(obj) => Type::Object(ObjectDetails {
                name: obj.name,
                fields: obj
                    .fields
                    .iter()
                    .map(|field| OutputField::from_parser(field, type_index))
                    .collect(),
            }),
            TypeDefinition::Interface(iface) => Type::Interface(InterfaceDetails {
                name: iface.name,
                fields: iface
                    .fields
                    .iter()
                    .map(|field| OutputField::from_parser(field, type_index))
                    .collect(),
            }),
            TypeDefinition::Union(union) => Type::Union(UnionDetails {
                name: union.name,
                types: union
                    .types
                    .iter()
                    .map(|name| OutputTypeRef::new(name, type_index))
                    .collect(),
            }),
            TypeDefinition::Enum(def) => Type::Enum(EnumDetails {
                name: def.name,
                values: def.values.iter().map(|v| v.name).collect(),
            }),
            TypeDefinition::InputObject(obj) => Type::InputObject(InputObjectDetails {
                name: obj.name,
                fields: obj
                    .fields
                    .iter()
                    .map(|field| InputField::from_parser(field, type_index))
                    .collect(),
            }),
        }
    }
}

impl<'schema> ScalarDetails<'schema> {
    pub fn is_builtin(&self) -> bool {
        match self.name {
            "String" | "Int" | "Boolean" | "ID" => true,
            _ => false,
        }
    }
}

impl<'schema> OutputType<'schema> {
    pub fn name(&self) -> &'schema str {
        match self {
            Self::Scalar(details) => details.name,
            Self::Object(details) => details.name,
            Self::Interface(details) => details.name,
            Self::Enum(details) => details.name,
            Self::Union(details) => details.name,
        }
    }
}

impl<'schema> TryFrom<Type<'schema>> for InputType<'schema> {
    type Error = Error;

    fn try_from(ty: Type<'schema>) -> Result<InputType<'schema>, Error> {
        match ty {
            Type::InputObject(inner) => Ok(InputType::InputObject(inner)),
            Type::Scalar(inner) => Ok(InputType::Scalar(inner)),
            Type::Enum(inner) => Ok(InputType::Enum(inner)),
            _ => Err(Error::ExpectedInputType),
        }
    }
}

impl<'schema> TryFrom<Type<'schema>> for OutputType<'schema> {
    type Error = Error;

    fn try_from(ty: Type<'schema>) -> Result<OutputType<'schema>, Error> {
        match ty {
            Type::Scalar(inner) => Ok(OutputType::Scalar(inner)),
            Type::Enum(inner) => Ok(OutputType::Enum(inner)),
            Type::Object(inner) => Ok(OutputType::Object(inner)),
            Type::Interface(inner) => Ok(OutputType::Interface(inner)),
            Type::Union(inner) => Ok(OutputType::Union(inner)),
            Type::InputObject(_) => Err(Error::ExpectedOutputType),
        }
    }
}
