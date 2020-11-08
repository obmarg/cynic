mod fields;
mod parser;
mod type_index;
mod type_refs;

pub use fields::*;
pub use parser::*;
pub use type_index::{GraphPath, TypeIndex};
pub use type_refs::{InputTypeRef, OutputTypeRef};

use std::{convert::TryFrom, rc::Rc};

use super::Error;

// TODO: Put these types somewhere sensible

enum Type<'schema> {
    Scalar(ScalarDetails<'schema>),
    Object(ObjectDetails<'schema>),
    Interface(InterfaceDetails<'schema>),
    Enum(EnumDetails<'schema>),
    Union(UnionDetails<'schema>),
    InputObject(InputObjectDetails<'schema>),
}

struct ScalarDetails<'schema> {
    name: &'schema str,
}

struct ObjectDetails<'schema> {
    name: &'schema str,
    fields: Vec<OutputField<'schema>>,
}

struct InterfaceDetails<'schema> {
    name: &'schema str,
    fields: Vec<OutputField<'schema>>,
}

struct EnumDetails<'schema> {
    name: &'schema str,
    values: Vec<&'schema str>,
}

struct UnionDetails<'schema> {
    name: &'schema str,
    types: Vec<OutputTypeRef<'schema>>,
}

struct InputObjectDetails<'schema> {
    name: &'schema str,
    fields: Vec<InputField<'schema>>,
}

pub enum InputType<'schema> {
    Scalar(ScalarDetails<'schema>),
    Enum(EnumDetails<'schema>),
    InputObject(InputObjectDetails<'schema>),
}

pub enum OutputType<'schema> {
    Scalar(ScalarDetails<'schema>),
    Object(ObjectDetails<'schema>),
    Interface(InterfaceDetails<'schema>),
    Enum(EnumDetails<'schema>),
    Union(UnionDetails<'schema>),
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
