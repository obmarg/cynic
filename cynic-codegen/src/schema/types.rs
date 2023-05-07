use std::{borrow::Cow, marker::PhantomData};

use super::{names::FieldName, SchemaError};

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct SchemaRoots<'a> {
    pub query: ObjectType<'a>,
    pub mutation: Option<ObjectType<'a>>,
    pub subscription: Option<ObjectType<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum Type<'a> {
    Scalar(ScalarType<'a>),
    Object(ObjectType<'a>),
    Interface(InterfaceType<'a>),
    Union(UnionType<'a>),
    Enum(EnumType<'a>),
    InputObject(InputObjectType<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum InputType<'a> {
    Scalar(ScalarType<'a>),
    Enum(EnumType<'a>),
    InputObject(InputObjectType<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub enum OutputType<'a> {
    Scalar(ScalarType<'a>),
    Object(ObjectType<'a>),
    Interface(InterfaceType<'a>),
    Union(UnionType<'a>),
    Enum(EnumType<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct ScalarType<'a> {
    #[with(rkyv::with::AsOwned)]
    pub name: Cow<'a, str>,
    pub builtin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct ObjectType<'a> {
    #[with(rkyv::with::AsOwned)]
    pub name: Cow<'a, str>,
    pub implements_interfaces: Vec<InterfaceRef<'a>>,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct Field<'a> {
    pub name: FieldName<'a>,
    pub arguments: Vec<InputValue<'a>>,
    pub field_type: TypeRef<'a, OutputType<'a>>,
    #[with(rkyv::with::AsOwned)]
    pub(super) parent_type_name: Cow<'a, str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct InterfaceType<'a> {
    #[with(rkyv::with::AsOwned)]
    pub name: Cow<'a, str>,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct UnionType<'a> {
    #[with(rkyv::with::AsOwned)]
    pub name: Cow<'a, str>,
    pub types: Vec<ObjectRef<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct EnumType<'a> {
    #[with(rkyv::with::AsOwned)]
    pub name: Cow<'a, str>,
    pub values: Vec<EnumValue<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct EnumValue<'a> {
    pub name: FieldName<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct InputObjectType<'a> {
    #[with(rkyv::with::AsOwned)]
    pub name: Cow<'a, str>,
    pub fields: Vec<InputValue<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
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

#[derive(Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct ObjectRef<'a>(#[with(rkyv::with::AsOwned)] pub(super) Cow<'a, str>);

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

#[derive(Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
pub struct InterfaceRef<'a>(#[with(rkyv::with::AsOwned)] pub(super) Cow<'a, str>);

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

#[derive(Clone, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes, bound(serialize = "__S: rkyv::ser::Serializer"))]
#[archive_attr(check_bytes(
    bound = "__C: rkyv::validation::ArchiveContext, <__C as rkyv::Fallible>::Error: rkyv::bytecheck::Error"
))]
pub enum TypeRef<'a, T> {
    Named(
        #[with(rkyv::with::AsOwned)] Cow<'a, str>,
        PhantomData<fn() -> T>,
    ),

    List(
        #[omit_bounds]
        #[archive_attr(omit_bounds)]
        Box<TypeRef<'a, T>>,
    ),
    Nullable(
        #[omit_bounds]
        #[archive_attr(omit_bounds)]
        Box<TypeRef<'a, T>>,
    ),
}

impl<'a, T> TypeRef<'a, T>
where
    Type<'a>: TryInto<T>,
    <Type<'a> as TryInto<T>>::Error: std::fmt::Debug,
{
    pub fn inner_type<SchemaState>(&self, schema: &'a super::Schema<'a, SchemaState>) -> T {
        match self {
            TypeRef::Named(name, _) => {
                // Note: We validate types prior to constructing a TypeRef
                // for them so the unsafe_lookup and unwrap here should
                // be safe.
                schema
                    .type_index
                    .unsafe_lookup(name)
                    .unwrap()
                    .try_into()
                    .unwrap()
            }
            TypeRef::List(inner) => inner.inner_type(schema),
            TypeRef::Nullable(inner) => inner.inner_type(schema),
        }
    }
}

impl std::fmt::Debug for TypeRef<'_, InputType<'_>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named(arg0, _) => f.debug_tuple("NamedInputType").field(arg0).finish(),
            Self::List(arg0) => f.debug_tuple("ListType").field(arg0).finish(),
            Self::Nullable(arg0) => f.debug_tuple("NullableType").field(arg0).finish(),
        }
    }
}

impl std::fmt::Debug for TypeRef<'_, OutputType<'_>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named(arg0, _) => f.debug_tuple("NamedOutputType").field(arg0).finish(),
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
