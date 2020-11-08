use std::rc::Rc;

use super::{parser, InputTypeRef, OutputTypeRef, TypeIndex};

/// A field on an output type i.e. an object or interface
pub struct OutputField<'schema> {
    name: &'schema str,
    value_type: OutputFieldType<'schema>,
    arguments: Vec<InputField<'schema>>,
}

/// A field on an input object
pub struct InputField<'schema> {
    name: &'schema str,
    value_type: InputFieldType<'schema>,
}

pub enum InputFieldType<'schema> {
    NamedType(InputTypeRef<'schema>),
    ListType(Box<InputFieldType<'schema>>),
    NonNullType(Box<InputFieldType<'schema>>),
}

pub enum OutputFieldType<'schema> {
    NamedType(OutputTypeRef<'schema>),
    ListType(Box<OutputFieldType<'schema>>),
    NonNullType(Box<OutputFieldType<'schema>>),
}

impl<'schema> OutputField<'schema> {
    pub(super) fn from_parser(
        field: &parser::Field<'schema>,
        type_index: &Rc<TypeIndex<'schema>>,
    ) -> OutputField<'schema> {
        OutputField {
            name: field.name,
            value_type: OutputFieldType::from_parser(&field.field_type, type_index),
            arguments: field
                .arguments
                .iter()
                .map(|arg| InputField::from_parser(arg, type_index))
                .collect(),
        }
    }
}

impl<'schema> InputField<'schema> {
    pub(super) fn from_parser(
        field: &parser::InputValue<'schema>,
        type_index: &Rc<TypeIndex<'schema>>,
    ) -> InputField<'schema> {
        InputField {
            name: field.name,
            value_type: InputFieldType::from_parser(&field.value_type, type_index),
        }
    }
}

macro_rules! impl_field_type_from_parser_type {
    ($target:ident, $ref_type:ident) => {
        impl<'schema> $target<'schema> {
            fn from_parser(
                parser_type: &parser::Type<'schema>,
                type_index: &Rc<TypeIndex<'schema>>,
            ) -> Self {
                use parser::Type;

                match parser_type {
                    Type::NamedType(name) => $target::NamedType($ref_type::new(name, type_index)),
                    Type::ListType(inner) => $target::ListType(Box::new($target::from_parser(
                        inner.as_ref(),
                        type_index,
                    ))),
                    Type::NonNullType(inner) => $target::NonNullType(Box::new(
                        $target::from_parser(inner.as_ref(), type_index),
                    )),
                }
            }
        }
    };
}

impl_field_type_from_parser_type!(InputFieldType, InputTypeRef);
impl_field_type_from_parser_type!(OutputFieldType, OutputTypeRef);
