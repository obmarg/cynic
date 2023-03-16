use std::{borrow::Cow, rc::Rc};

use {
    super::{parser, InputTypeRef, OutputTypeRef, TypeIndex},
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

    /// Second returned type is whether we should generate a `'a` lifetime on
    /// the struct
    pub fn type_spec(
        &self,
        needs_boxed: bool,
        needs_owned: bool,
        is_subobject_with_lifetime: bool,
    ) -> TypeSpec {
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
        return input_type_spec_imp(
            ty,
            false,
            needs_boxed,
            needs_owned,
            is_subobject_with_lifetime,
        )
        .map(|type_spec| format!("Option<{type_spec}>",));
    }

    match ty {
        InputFieldType::ListType(inner) => {
            input_type_spec_imp(inner, true, false, needs_owned, is_subobject_with_lifetime)
                .map(|type_spec| format!("Vec<{type_spec}>",))
        }

        InputFieldType::NonNullType(_) => panic!("NonNullType somehow got past an if let"),

        InputFieldType::NamedType(s) => {
            let mut contains_lifetime_a = false;
            let mut name = match (s.type_name.as_ref(), needs_owned) {
                ("Int", _) => Cow::Borrowed("i32"),
                ("Float", _) => Cow::Borrowed("f64"),
                ("Boolean", _) => Cow::Borrowed("bool"),
                ("ID", true) => Cow::Borrowed("cynic::Id"),
                ("ID", false) => {
                    contains_lifetime_a = true;
                    Cow::Borrowed("&'a cynic::Id")
                }
                ("String", false) => {
                    contains_lifetime_a = true;
                    Cow::Borrowed("&'a str")
                }
                _ => Cow::Owned({
                    let mut type_ = s.type_name.to_pascal_case();
                    if is_subobject_with_lifetime {
                        type_ += "<'a>";
                        contains_lifetime_a = true;
                    }
                    type_
                }),
            };

            if needs_boxed {
                name = Cow::Owned(format!("Box<{}>", name));
            }

            TypeSpec {
                name,
                contains_lifetime_a,
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
