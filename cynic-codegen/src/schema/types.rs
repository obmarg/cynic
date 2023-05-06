use std::{borrow::Cow, marker::PhantomData};

use super::{names::FieldName, type_index::TypeIndex, SchemaError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type<'a> {
    Scalar(ScalarType<'a>),
    Object(ObjectType<'a>),
    Interface(InterfaceType<'a>),
    Union(UnionType<'a>),
    Enum(EnumType<'a>),
    InputObject(InputObjectType<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputType<'a> {
    Scalar(ScalarType<'a>),
    Enum(EnumType<'a>),
    InputObject(InputObjectType<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OutputType<'a> {
    Scalar(ScalarType<'a>),
    Object(ObjectType<'a>),
    Interface(InterfaceType<'a>),
    Union(UnionType<'a>),
    Enum(EnumType<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScalarType<'a> {
    pub name: Cow<'a, str>,
    pub builtin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectType<'a> {
    pub description: Option<&'a str>,
    pub name: Cow<'a, str>,
    pub implements_interfaces: Vec<InterfaceRef<'a>>,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field<'a> {
    pub name: FieldName<'a>,
    pub arguments: Vec<InputValue<'a>>,
    pub field_type: TypeRef<'a, OutputType<'a>>,
    pub(super) parent_type_name: Cow<'a, str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputValue<'a> {
    pub name: FieldName<'a>,
    pub value_type: TypeRef<'a, InputType<'a>>,
    pub has_default: bool,
}

impl InputValue<'_> {
    pub fn is_nullable(&self) -> bool {
        matches!(self.value_type, TypeRef::Nullable(_))
    }

    pub fn is_required(&self) -> bool {
        !(self.has_default || matches!(self.value_type, TypeRef::Nullable(_)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterfaceType<'a> {
    pub name: Cow<'a, str>,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionType<'a> {
    pub name: Cow<'a, str>,
    pub types: Vec<ObjectRef<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumType<'a> {
    pub name: Cow<'a, str>,
    pub values: Vec<EnumValue<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumValue<'a> {
    pub name: FieldName<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputObjectType<'a> {
    pub name: Cow<'a, str>,
    pub fields: Vec<InputValue<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    InputType,
    OutputType,
    Object,
    Scalar,
    Interface,
    Union,
    Enum,
    InputObject,
    ObjectOrInterface,
    UnionOrInterface,
}

impl<'a> Type<'a> {
    pub fn object(&self) -> Option<&ObjectType<'a>> {
        match self {
            Type::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

impl<'a> ObjectType<'a> {
    pub fn field<N>(&self, name: &N) -> Option<&Field<'a>>
    where
        N: ?Sized,
        for<'b> FieldName<'b>: PartialEq<N>,
    {
        self.fields.iter().find(|field| field.name == *name)
    }
}

impl<'a> InputObjectType<'a> {
    pub fn field<N>(&self, name: &N) -> Option<&InputValue<'a>>
    where
        for<'b> FieldName<'b>: PartialEq<N>,
    {
        self.fields.iter().find(|field| field.name == *name)
    }
}

impl<'a> EnumType<'a> {
    pub fn value<N>(&self, name: &N) -> Option<&EnumValue<'a>>
    where
        N: ?Sized,
        for<'b> FieldName<'b>: PartialEq<N>,
    {
        self.values.iter().find(|value| value.name == *name)
    }
}

#[derive(Clone)]
pub struct ObjectRef<'a>(pub(super) Cow<'a, str>, pub(super) TypeIndex<'a>);

impl std::fmt::Debug for ObjectRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ObjectRef").field(&self.0).finish()
    }
}

impl<'a> PartialEq for ObjectRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ObjectRef<'_> {}

impl<'a> std::hash::Hash for ObjectRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Clone)]
pub struct InterfaceRef<'a>(pub(super) Cow<'a, str>, pub(super) TypeIndex<'a>);

impl std::fmt::Debug for InterfaceRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InterfaceRef").field(&self.0).finish()
    }
}

impl<'a> PartialEq for InterfaceRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for InterfaceRef<'_> {}

impl<'a> std::hash::Hash for InterfaceRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Clone)]
pub enum TypeRef<'a, T> {
    Named(Cow<'a, str>, TypeIndex<'a>, PhantomData<fn() -> T>),
    List(Box<TypeRef<'a, T>>),
    Nullable(Box<TypeRef<'a, T>>),
}


impl<T> PartialEq for TypeRef<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Named(l0, _, _), Self::Named(r0, _, _)) => l0 == r0,
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::Nullable(l0), Self::Nullable(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl<T> Eq for TypeRef<'_, T> {}

impl<T> std::hash::Hash for TypeRef<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TypeRef::Named(inner, _, _) => inner.hash(state),
            TypeRef::List(inner) => {
                1.hash(state);
                inner.hash(state);
            }
            TypeRef::Nullable(inner) => {
                2.hash(state);
                inner.hash(state)
            }
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

impl<'a> TryFrom<Type<'a>> for OutputType<'a> {
    type Error = SchemaError;

    fn try_from(value: Type<'a>) -> Result<Self, Self::Error> {
        match value {
            Type::Scalar(inner) => Ok(OutputType::Scalar(inner)),
            Type::Object(inner) => Ok(OutputType::Object(inner)),
            Type::Interface(inner) => Ok(OutputType::Interface(inner)),
            Type::Union(inner) => Ok(OutputType::Union(inner)),
            Type::Enum(inner) => Ok(OutputType::Enum(inner)),
            Type::InputObject(inner) => Err(SchemaError::UnexpectedKind {
                name: inner.name.to_string(),
                expected: Kind::OutputType,
                found: Kind::InputObject,
            }),
        }
    }
}

impl<'a> TryFrom<Type<'a>> for InputType<'a> {
    type Error = SchemaError;

    fn try_from(value: Type<'a>) -> Result<Self, Self::Error> {
        match value {
            Type::Scalar(inner) => Ok(InputType::Scalar(inner)),
            Type::InputObject(inner) => Ok(InputType::InputObject(inner)),
            Type::Object(inner) => Err(SchemaError::UnexpectedKind {
                name: inner.name.to_string(),
                expected: Kind::InputType,
                found: Kind::Object,
            }),
            Type::Enum(inner) => Ok(InputType::Enum(inner)),
            _ => Err(SchemaError::unexpected_kind(value, Kind::InputType)),
        }
    }
}

impl<'a> TryFrom<Type<'a>> for ObjectType<'a> {
    type Error = SchemaError;

    fn try_from(value: Type<'a>) -> Result<Self, Self::Error> {
        match value {
            Type::Object(inner) => Ok(inner),
            _ => Err(SchemaError::unexpected_kind(value, Kind::Object)),
        }
    }
}

impl<'a> TryFrom<Type<'a>> for InputObjectType<'a> {
    type Error = SchemaError;

    fn try_from(value: Type<'a>) -> Result<Self, Self::Error> {
        match value {
            Type::InputObject(inner) => Ok(inner),
            _ => Err(SchemaError::unexpected_kind(value, Kind::InputObject)),
        }
    }
}

impl<'a> TryFrom<Type<'a>> for EnumType<'a> {
    type Error = SchemaError;

    fn try_from(value: Type<'a>) -> Result<Self, Self::Error> {
        match value {
            Type::Enum(inner) => Ok(inner),
            _ => Err(SchemaError::unexpected_kind(value, Kind::Enum)),
        }
    }
}

impl<'a> Type<'a> {
    pub fn name(&'a self) -> &'a str {
        match self {
            Type::Scalar(inner) => inner.name.as_ref(),
            Type::Object(inner) => inner.name.as_ref(),
            Type::Interface(inner) => inner.name.as_ref(),
            Type::Union(inner) => inner.name.as_ref(),
            Type::Enum(inner) => inner.name.as_ref(),
            Type::InputObject(inner) => inner.name.as_ref(),
        }
    }

    pub fn kind(&self) -> Kind {
        match self {
            Type::Scalar(_) => Kind::Scalar,
            Type::Object(_) => Kind::Object,
            Type::Interface(_) => Kind::Interface,
            Type::Union(_) => Kind::Union,
            Type::Enum(_) => Kind::Enum,
            Type::InputObject(_) => Kind::InputObject,
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Kind::InputType => "input type",
            Kind::OutputType => "output type",
            Kind::Object => "object",
            Kind::Scalar => "scalar",
            Kind::Interface => "interface",
            Kind::Union => "union",
            Kind::Enum => "enum",
            Kind::InputObject => "input object",
            Kind::ObjectOrInterface => "object or interface",
            Kind::UnionOrInterface => "union or interface",
        };
        write!(f, "{}", s)
    }
}
