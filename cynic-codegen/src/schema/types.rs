use std::marker::PhantomData;

use super::type_index::TypeIndex;

#[derive(Debug, Clone, PartialEq)]
pub enum Type<'a> {
    Scalar(ScalarType<'a>),
    Object(ObjectType<'a>),
    Interface(InterfaceType<'a>),
    Union(UnionType<'a>),
    Enum(EnumType<'a>),
    InputObject(InputObjectType<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputType<'a> {
    Scalar(ScalarType<'a>),
    Enum(EnumType<'a>),
    InputObject(InputObjectType<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputType<'a> {
    Scalar(ScalarType<'a>),
    Object(ObjectType<'a>),
    Interface(InterfaceType<'a>),
    Union(UnionType<'a>),
    Enum(EnumType<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarType<'a> {
    pub name: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectType<'a> {
    pub description: Option<&'a str>,
    pub name: &'a str,
    // pub implements_interfaces: Vec<InterfaceRef>,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field<'a> {
    pub description: Option<&'a str>,
    pub name: &'a str,
    pub arguments: Vec<InputValue<'a>>,
    pub field_type: TypeRef<'a, OutputType<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputValue<'a> {
    pub description: Option<&'a str>,
    pub name: &'a str,
    pub value_type: TypeRef<'a, InputType<'a>>,
    // pub default_value: Option<Value<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceType<'a> {
    pub description: Option<&'a str>,
    pub name: &'a str,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionType<'a> {
    pub description: Option<&'a str>,
    pub name: &'a str,
    pub types: Vec<&'a str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumType<'a> {
    pub description: Option<&'a str>,
    pub name: &'a str,
    pub values: Vec<EnumValue<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue<'a> {
    pub description: Option<&'a str>,
    pub name: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputObjectType<'a> {
    pub description: Option<&'a str>,
    pub name: &'a str,
    pub fields: Vec<InputValue<'a>>,
}

#[derive(Clone)]
pub enum TypeRef<'a, T> {
    Named(&'a str, &'a TypeIndex<'a>, PhantomData<fn() -> T>),
    List(Box<TypeRef<'a, T>>),
    Nullable(Box<TypeRef<'a, T>>),
}

impl<'a, T> PartialEq for TypeRef<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Named(l0, l1, _), Self::Named(r0, r1, _)) => l0 == r0 && std::ptr::eq(l1, r1),
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::Nullable(l0), Self::Nullable(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl std::fmt::Debug for TypeRef<'_, InputType<'_>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named(arg0, _, _) => f.debug_tuple("NamedInputType").field(arg0).finish(),
            Self::List(arg0) => f.debug_tuple("ListType").field(arg0).finish(),
            Self::Nullable(arg0) => f.debug_tuple("NullableType").field(arg0).finish(),
        }
    }
}

impl std::fmt::Debug for TypeRef<'_, OutputType<'_>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named(arg0, _, _) => f.debug_tuple("NamedOutputType").field(arg0).finish(),
            Self::List(arg0) => f.debug_tuple("ListType").field(arg0).finish(),
            Self::Nullable(arg0) => f.debug_tuple("NullableType").field(arg0).finish(),
        }
    }
}
