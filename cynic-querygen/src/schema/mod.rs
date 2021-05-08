mod fields;
mod parser;
mod type_index;
mod type_refs;

pub use fields::*;
pub use parser::*;
pub use type_index::{GraphPath, TypeIndex};
pub use type_refs::{InputTypeRef, InterfaceTypeRef, OutputTypeRef, TypeRef};

use std::{convert::TryFrom, rc::Rc};

use crate::Error;

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
    pub name: &'schema str,
    pub fields: Vec<OutputField<'schema>>,
    implements_interfaces: Vec<InterfaceTypeRef<'schema>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InterfaceDetails<'schema> {
    pub name: &'schema str,
    pub fields: Vec<OutputField<'schema>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EnumDetails<'schema> {
    pub name: &'schema str,
    pub values: Vec<&'schema str>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnionDetails<'schema> {
    pub name: &'schema str,
    pub types: Vec<OutputTypeRef<'schema>>,
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
pub struct InterfaceType<'schema>(pub InterfaceDetails<'schema>);

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
                implements_interfaces: obj
                    .implements_interfaces
                    .iter()
                    .map(|name| InterfaceTypeRef::new(name, type_index))
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

    pub fn name(&self) -> &'schema str {
        match self {
            Self::Scalar(details) => details.name,
            Self::Object(details) => details.name,
            Self::Interface(details) => details.name,
            Self::Enum(details) => details.name,
            Self::Union(details) => details.name,
            Self::InputObject(details) => details.name,
        }
    }

    pub fn allows_fragment_target_of(&self, target: &Type<'schema>) -> Result<(), Error> {
        match self {
            Type::Interface(iface) => {
                if let Type::Object(obj) = target {
                    if obj.implements_interface(iface) {
                        return Ok(());
                    }
                }
                return Err(Error::TypeDoesNotImplementInterface(
                    target.name().to_string(),
                    iface.name.to_string(),
                ));
            }
            Type::Union(details) => {
                if details.has_member(target) {
                    return Ok(());
                }

                return Err(Error::TypeNotUnionMember(
                    target.name().to_string(),
                    self.name().to_string(),
                ));
            }
            _ => {
                return Err(Error::InlineFragmentOnUnsupportedType(
                    self.name().to_string(),
                ))
            }
        }
    }
}

impl<'schema> ScalarDetails<'schema> {
    pub fn is_builtin(&self) -> bool {
        matches!(self.name, "String" | "Int" | "Boolean" | "ID")
    }
}

impl<'schema> ObjectDetails<'schema> {
    fn implements_interface(&self, interface: &InterfaceDetails<'schema>) -> bool {
        self.implements_interfaces
            .iter()
            .any(|iface_ref| iface_ref.lookup().ok().map(|i| i.0.name) == Some(interface.name))
    }
}

impl<'schema> UnionDetails<'schema> {
    fn has_member(&self, member: &Type<'schema>) -> bool {
        self.types
            .iter()
            .any(|type_ref| type_ref.lookup().ok().map(|t| t.name()) == Some(member.name()))
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

impl<'schema> InputType<'schema> {
    /// Checks whether it's safe to assume an InputType is Copy
    ///
    /// This might give false negatives for user defined scalars,
    /// but not much that can be done about that...
    pub fn is_definitely_copy(&self) -> bool {
        match self {
            InputType::InputObject(_) => false,
            InputType::Enum(_) => true,
            InputType::Scalar(details) => details.name == "Int" || details.name == "Boolean",
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

impl<'schema> TryFrom<Type<'schema>> for InterfaceType<'schema> {
    type Error = Error;

    fn try_from(ty: Type<'schema>) -> Result<InterfaceType<'schema>, Error> {
        match ty {
            Type::Interface(inner) => Ok(InterfaceType(inner)),
            _ => Err(Error::ExpectedInterfaceType),
        }
    }
}
