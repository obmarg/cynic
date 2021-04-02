use std::{borrow::Cow, rc::Rc};

use super::{parser, InputType, InputTypeRef, OutputTypeRef, TypeIndex};
use crate::Error;

/// A field on an output type i.e. an object or interface
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct OutputField<'schema> {
    pub name: &'schema str,
    pub value_type: OutputFieldType<'schema>,
    pub arguments: Vec<InputField<'schema>>,
}

/// A field on an input object or an argument
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct InputField<'schema> {
    pub name: &'schema str,
    pub value_type: InputFieldType<'schema>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum InputFieldType<'schema> {
    NamedType(InputTypeRef<'schema>),
    ListType(Box<InputFieldType<'schema>>),
    NonNullType(Box<InputFieldType<'schema>>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
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

impl<'schema> OutputFieldType<'schema> {
    pub fn inner_name(&self) -> Cow<'schema, str> {
        match self {
            OutputFieldType::NamedType(name) => name.type_name.clone(),
            OutputFieldType::NonNullType(inner) => inner.inner_name(),
            OutputFieldType::ListType(inner) => inner.inner_name(),
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

    pub fn type_spec(&self) -> Cow<'schema, str> {
        self.value_type.type_spec()
    }
}

impl<'schema> InputFieldType<'schema> {
    pub fn from_variable_definition<'query>(
        def: &graphql_parser::query::VariableDefinition<'query, &'query str>,
        type_index: &Rc<TypeIndex<'schema>>,
    ) -> Self {
        InputFieldType::from_query_type(&def.var_type, type_index)
    }

    fn from_query_type<'query>(
        query_type: &graphql_parser::query::Type<'query, &'query str>,
        type_index: &Rc<TypeIndex<'schema>>,
    ) -> Self {
        use parser::Type;

        match query_type {
            Type::NamedType(name) => {
                InputFieldType::NamedType(InputTypeRef::new_owned(name.to_string(), type_index))
            }
            Type::ListType(inner) => InputFieldType::ListType(Box::new(Self::from_query_type(
                inner.as_ref(),
                type_index,
            ))),
            Type::NonNullType(inner) => InputFieldType::NonNullType(Box::new(
                Self::from_query_type(inner.as_ref(), type_index),
            )),
        }
    }

    pub fn inner_name(&self) -> Cow<'schema, str> {
        match self {
            InputFieldType::NamedType(name) => name.type_name.clone(),
            InputFieldType::NonNullType(inner) => inner.inner_name(),
            InputFieldType::ListType(inner) => inner.inner_name(),
        }
    }

    // Gets the inner InputFieldType of a list, if this type _is_ a list.
    pub fn list_inner_type<'a>(&'a self) -> Result<&InputFieldType<'schema>, Error> {
        match self {
            InputFieldType::NonNullType(inner) => inner.list_inner_type(),
            InputFieldType::NamedType(_) => Err(Error::ExpectedListType),
            InputFieldType::ListType(inner) => Ok(inner.as_ref()),
        }
    }

    pub fn contains_list(&self) -> bool {
        match self {
            InputFieldType::NonNullType(inner) => inner.contains_list(),
            InputFieldType::NamedType(_) => false,
            InputFieldType::ListType(_) => true,
        }
    }

    pub fn type_spec(&self) -> Cow<'schema, str> {
        input_type_spec_imp(&self, true)
    }
}

fn input_type_spec_imp<'schema>(ty: &InputFieldType<'schema>, nullable: bool) -> Cow<'schema, str> {
    use inflector::Inflector;

    if let InputFieldType::NonNullType(inner) = ty {
        return input_type_spec_imp(inner, false);
    }

    if nullable {
        return Cow::Owned(format!("Option<{}>", input_type_spec_imp(ty, false)));
    }

    match ty {
        InputFieldType::ListType(inner) => {
            Cow::Owned(format!("Vec<{}>", input_type_spec_imp(inner, true)))
        }

        InputFieldType::NonNullType(_) => panic!("NonNullType somehow got past an if let"),

        InputFieldType::NamedType(s) => {
            match s.type_name.as_ref() {
                "Int" => return Cow::Borrowed("i32"),
                "Float" => return Cow::Borrowed("f64"),
                "Boolean" => return Cow::Borrowed("bool"),
                "ID" => return Cow::Borrowed("cynic::Id"),
                _ => {}
            }

            match s.lookup() {
                Ok(InputType::Enum(_)) => Cow::Owned(s.type_name.to_pascal_case()),
                Ok(InputType::InputObject(_)) => Cow::Owned(s.type_name.to_pascal_case()),
                _ => s.type_name.clone(),
            }
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

macro_rules! impl_inner_ref {
    ($target:ident, $inner_type:ident) => {
        impl<'schema> $target<'schema> {
            pub fn inner_ref(&self) -> &$inner_type<'schema> {
                match self {
                    $target::NamedType(inner) => inner,
                    $target::NonNullType(inner) => inner.inner_ref(),
                    $target::ListType(inner) => inner.inner_ref(),
                }
            }
        }
    };
}

impl_inner_ref!(InputFieldType, InputTypeRef);
impl_inner_ref!(OutputFieldType, OutputTypeRef);
