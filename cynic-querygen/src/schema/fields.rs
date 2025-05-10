use std::{borrow::Cow, rc::Rc};

use cynic_parser::type_system as parser;
use {
    super::{InputTypeRef, OutputTypeRef, TypeIndex},
    crate::Error,
};

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
#[allow(clippy::enum_variant_names)]
pub enum InputFieldType<'schema> {
    NamedType(InputTypeRef<'schema>),
    ListType(Box<InputFieldType<'schema>>),
    NonNullType(Box<InputFieldType<'schema>>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum OutputFieldType<'schema> {
    NamedType(OutputTypeRef<'schema>),
    ListType(Box<OutputFieldType<'schema>>),
    NonNullType(Box<OutputFieldType<'schema>>),
}

impl<'schema> OutputField<'schema> {
    pub(super) fn from_parser(
        field: cynic_parser::type_system::FieldDefinition<'schema>,
        type_index: &Rc<TypeIndex<'schema>>,
    ) -> OutputField<'schema> {
        OutputField {
            name: field.name(),
            value_type: OutputFieldType::from_parser(field.ty(), type_index),
            arguments: field
                .arguments()
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
        field: parser::InputValueDefinition<'schema>,
        type_index: &Rc<TypeIndex<'schema>>,
    ) -> InputField<'schema> {
        InputField {
            name: field.name(),
            value_type: InputFieldType::from_parser(field.ty(), type_index),
        }
    }
}

impl<'schema> InputFieldType<'schema> {
    pub fn from_variable_definition(
        def: cynic_parser::executable::VariableDefinition<'schema>,
        type_index: &Rc<TypeIndex<'schema>>,
    ) -> Self {
        InputFieldType::from_query_type(&def.ty(), type_index)
    }

    fn from_query_type(
        query_type: &cynic_parser::executable::Type<'schema>,
        type_index: &Rc<TypeIndex<'schema>>,
    ) -> Self {
        use cynic_parser::common::WrappingType;

        let mut ty = InputFieldType::NamedType(InputTypeRef::new(query_type.name(), type_index));
        for wrapping in query_type.wrappers().collect::<Vec<_>>().into_iter().rev() {
            match wrapping {
                WrappingType::NonNull => ty = InputFieldType::NonNullType(Box::new(ty)),
                WrappingType::List => ty = InputFieldType::ListType(Box::new(ty)),
            }
        }
        ty
    }

    pub fn inner_name(&self) -> Cow<'schema, str> {
        match self {
            InputFieldType::NamedType(name) => name.type_name.clone(),
            InputFieldType::NonNullType(inner) => inner.inner_name(),
            InputFieldType::ListType(inner) => inner.inner_name(),
        }
    }

    // Gets the inner InputFieldType of a list, if this type _is_ a list.
    pub fn list_inner_type<'a>(&'a self) -> Result<&'a InputFieldType<'schema>, Error> {
        match self {
            InputFieldType::NonNullType(inner) => inner.list_inner_type(),
            InputFieldType::NamedType(_) => Err(Error::ExpectedListType),
            InputFieldType::ListType(inner) => Ok(inner.as_ref()),
        }
    }

    /// Second returned type is whether we should generate a `'a` lifetime on
    /// the struct
    pub fn type_spec(
        &self,
        needs_boxed: bool,
        needs_owned: bool,
        is_subobject_with_lifetime: bool,
    ) -> TypeSpec<'static> {
        input_type_spec_imp(
            self,
            true,
            needs_boxed,
            needs_owned,
            is_subobject_with_lifetime,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeSpec<'a> {
    pub(crate) name: Cow<'a, str>,
    pub(crate) contains_lifetime_a: bool,
}

impl<'a> TypeSpec<'a> {
    fn map(self, f: impl FnOnce(&str) -> String) -> TypeSpec<'static> {
        TypeSpec {
            name: Cow::Owned(f(&self.name)),
            contains_lifetime_a: self.contains_lifetime_a,
        }
    }
    pub(crate) fn lifetime<'b>(
        struct_type_specs: impl IntoIterator<Item = &'b Self>,
    ) -> &'static str
    where
        'a: 'b,
    {
        if struct_type_specs
            .into_iter()
            .any(|ts| ts.contains_lifetime_a)
        {
            "<'a>"
        } else {
            ""
        }
    }
}

/// Second returned type is whether we should generate a `'a` lifetime on the
/// struct
fn input_type_spec_imp(
    ty: &InputFieldType<'_>,
    nullable: bool,
    needs_boxed: bool,
    needs_owned: bool,
    is_subobject_with_lifetime: bool,
) -> TypeSpec<'static> {
    use crate::casings::CasingExt;

    if let InputFieldType::NonNullType(inner) = ty {
        return input_type_spec_imp(
            inner,
            false,
            needs_boxed,
            needs_owned,
            is_subobject_with_lifetime,
        );
    }

    if nullable {
        let inner_spec = input_type_spec_imp(
            ty,
            false,
            needs_boxed,
            needs_owned,
            is_subobject_with_lifetime,
        );
        return TypeSpec {
            name: Cow::Owned(format!("Option<{}>", inner_spec.name)),
            contains_lifetime_a: inner_spec.contains_lifetime_a,
        };
    }

    match ty {
        InputFieldType::ListType(inner) => input_type_spec_imp(
            inner,
            true,
            needs_boxed,
            needs_owned,
            is_subobject_with_lifetime,
        )
        .map(|type_spec| format!("Vec<{type_spec}>",)),
        InputFieldType::NonNullType(_) => panic!("NonNullType somehow got past an if let"),
        InputFieldType::NamedType(s) => {
            let mut base_type_name_str: String;
            let mut base_type_contains_lifetime = false;

            match s.type_name.as_ref() {
                "Int" => base_type_name_str = "i32".to_string(),
                "Float" => base_type_name_str = "f64".to_string(),
                "Boolean" => base_type_name_str = "bool".to_string(),
                "ID" => {
                    if needs_owned {
                        base_type_name_str = "cynic::Id".to_string();
                    } else {
                        base_type_name_str = "&'a cynic::Id".to_string();
                        base_type_contains_lifetime = true;
                    }
                }
                "String" => {
                    if needs_owned {
                        base_type_name_str = "String".to_string();
                    } else {
                        base_type_name_str = "&'a str".to_string();
                        base_type_contains_lifetime = true;
                    }
                }
                _ => {
                    base_type_name_str = s.type_name.to_pascal_case();
                    if is_subobject_with_lifetime {
                        base_type_name_str = format!("{}<'a>", base_type_name_str);
                        base_type_contains_lifetime = true;
                    }
                }
            };

            let final_name_str = if needs_boxed {
                format!("Box<{}>", base_type_name_str)
            } else {
                base_type_name_str
            };

            TypeSpec {
                name: Cow::Owned(final_name_str),
                contains_lifetime_a: base_type_contains_lifetime,
            }
        }
    }
}

macro_rules! impl_field_type_from_parser_type {
    ($target:ident, $ref_type:ident) => {
        impl<'schema> $target<'schema> {
            pub fn from_parser(
                parser_type: parser::Type<'schema>,
                type_index: &Rc<TypeIndex<'schema>>,
            ) -> Self {
                use cynic_parser::common::WrappingType;

                let mut ty = $target::NamedType($ref_type::new(parser_type.name(), type_index));
                for wrapping in parser_type.wrappers().collect::<Vec<_>>().into_iter().rev() {
                    match wrapping {
                        WrappingType::NonNull => ty = $target::NonNullType(Box::new(ty)),
                        WrappingType::List => ty = $target::ListType(Box::new(ty)),
                    }
                }
                ty
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
