pub use crate::query::DirectiveLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Schema {
    pub query_type: String,
    pub mutation_type: Option<String>,
    pub subscription_type: Option<String>,
    pub types: Vec<Type>,
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Directive {
    pub name: String,
    pub description: Option<String>,
    pub args: Vec<InputValue>,
    pub locations: Vec<DirectiveLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Type {
    Object(ObjectType),
    InputObject(InputObjectType),
    Enum(EnumType),
    Interface(InterfaceType),
    Union(UnionType),
    Scalar(ScalarType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectType {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<Field>,
    pub interfaces: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct InputObjectType {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<InputValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct EnumType {
    pub name: String,
    pub description: Option<String>,
    pub values: Vec<EnumValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct InterfaceType {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<Field>,
    pub interfaces: Vec<String>,
    pub possible_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnionType {
    pub name: String,
    pub description: Option<String>,
    pub possible_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ScalarType {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct EnumValue {
    pub name: String,
    pub description: Option<String>,
    pub deprecated: Deprecated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Deprecated {
    No,
    Yes(Option<String>),
}

impl Deprecated {
    fn new(is_deprecated: bool, deprecation_reason: Option<String>) -> Deprecated {
        match (is_deprecated, deprecation_reason) {
            (false, _) => Deprecated::No,
            (true, reason) => Deprecated::Yes(reason),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub args: Vec<InputValue>,
    pub ty: FieldType,
    pub deprecated: Deprecated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]

pub struct InputValue {
    pub name: String,
    pub description: Option<String>,
    pub ty: FieldType,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FieldType {
    pub wrapping: FieldWrapping,
    pub name: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FieldWrapping([u8; 8]);

impl std::fmt::Debug for FieldWrapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.into_iter().collect::<Vec<_>>())
    }
}

impl IntoIterator for FieldWrapping {
    type Item = WrappingType;

    type IntoIter = Box<dyn Iterator<Item = WrappingType>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            self.0
                .into_iter()
                .take_while(|&item| item != 0)
                .flat_map(|item| match item {
                    1 => std::iter::once(WrappingType::List),
                    2 => std::iter::once(WrappingType::NonNull),
                    _ => unreachable!(),
                }),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrappingType {
    List,
    NonNull,
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
pub enum SchemaError {
    #[error("A type in the introspection output was missing a name")]
    TypeMissingName,
    #[error("Found a list wrapper type in a position that should contain a named type")]
    UnexpectedList,
    #[error("Found a non-null wrapper type in a position that should contain a named type")]
    UnexpectedNonNull,
    #[error("Found a wrapping type that had no inner ofType")]
    WrappingTypeWithNoInner,
    #[error("Found a wrapping type that was too nested")]
    TooMuchWrapping,
}

impl TryFrom<crate::query::Schema> for Schema {
    type Error = SchemaError;

    fn try_from(schema: crate::query::Schema) -> Result<Self, Self::Error> {
        Ok(Schema {
            query_type: schema.query_type.into_name()?,
            mutation_type: schema.mutation_type.map(|ty| ty.into_name()).transpose()?,
            subscription_type: schema
                .subscription_type
                .map(|ty| ty.into_name())
                .transpose()?,
            types: schema
                .types
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            directives: schema
                .directives
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<crate::query::Type> for Type {
    type Error = SchemaError;

    fn try_from(ty: crate::query::Type) -> Result<Self, Self::Error> {
        match ty.kind {
            crate::query::TypeKind::Scalar => Ok(Type::Scalar(ScalarType {
                name: ty.name.ok_or(SchemaError::TypeMissingName)?,
                description: ty.description,
            })),
            crate::query::TypeKind::Object => Ok(Type::Object(ObjectType {
                name: ty.name.ok_or(SchemaError::TypeMissingName)?,
                fields: ty
                    .fields
                    .unwrap_or_default()
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
                description: ty.description,
                interfaces: ty
                    .interfaces
                    .unwrap_or_default()
                    .into_iter()
                    .map(|ty| ty.into_name())
                    .collect::<Result<Vec<_>, _>>()?,
            })),
            crate::query::TypeKind::Interface => Ok(Type::Interface(InterfaceType {
                name: ty.name.ok_or(SchemaError::TypeMissingName)?,
                description: ty.description,
                fields: ty
                    .fields
                    .unwrap_or_default()
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
                interfaces: ty
                    .interfaces
                    .unwrap_or_default()
                    .into_iter()
                    .map(|ty| ty.into_name())
                    .collect::<Result<Vec<_>, _>>()?,
                possible_types: ty
                    .possible_types
                    .unwrap_or_default()
                    .into_iter()
                    .map(|ty| ty.into_name())
                    .collect::<Result<Vec<_>, _>>()?,
            })),
            crate::query::TypeKind::Union => Ok(Type::Union(UnionType {
                name: ty.name.ok_or(SchemaError::TypeMissingName)?,
                description: ty.description,
                possible_types: ty
                    .possible_types
                    .unwrap_or_default()
                    .into_iter()
                    .map(|ty| ty.into_name())
                    .collect::<Result<Vec<_>, _>>()?,
            })),
            crate::query::TypeKind::Enum => Ok(Type::Enum(EnumType {
                name: ty.name.ok_or(SchemaError::TypeMissingName)?,
                description: ty.description,
                values: ty
                    .enum_values
                    .unwrap_or_default()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            })),
            crate::query::TypeKind::InputObject => Ok(Type::InputObject(InputObjectType {
                name: ty.name.ok_or(SchemaError::TypeMissingName)?,
                description: ty.description,
                fields: ty
                    .input_fields
                    .unwrap_or_default()
                    .into_iter()
                    .map(InputValue::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            })),
            crate::query::TypeKind::List => Err(SchemaError::UnexpectedList),
            crate::query::TypeKind::NonNull => Err(SchemaError::UnexpectedNonNull),
        }
    }
}

impl TryFrom<crate::query::Field> for Field {
    type Error = SchemaError;

    fn try_from(field: crate::query::Field) -> Result<Self, Self::Error> {
        Ok(Field {
            name: field.name,
            description: field.description,
            args: field
                .args
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            ty: field.ty.try_into()?,
            deprecated: Deprecated::new(field.is_deprecated, field.deprecation_reason),
        })
    }
}

impl TryFrom<crate::query::FieldType> for FieldType {
    type Error = SchemaError;

    fn try_from(field_type: crate::query::FieldType) -> Result<Self, Self::Error> {
        let mut wrapping = [0; 8];
        let mut wrapping_pos = 0;
        let mut current_ty = field_type;
        loop {
            if wrapping_pos >= wrapping.len() {
                return Err(SchemaError::TooMuchWrapping);
            }
            match current_ty.kind {
                crate::query::TypeKind::List => {
                    wrapping[wrapping_pos] = 1;
                    wrapping_pos += 1;
                    current_ty = *current_ty
                        .of_type
                        .ok_or(SchemaError::WrappingTypeWithNoInner)?;
                }
                crate::query::TypeKind::NonNull => {
                    wrapping[wrapping_pos] = 2;
                    wrapping_pos += 1;
                    current_ty = *current_ty
                        .of_type
                        .ok_or(SchemaError::WrappingTypeWithNoInner)?;
                }
                _ => {
                    return Ok(FieldType {
                        name: current_ty.name.ok_or(SchemaError::TypeMissingName)?,
                        wrapping: FieldWrapping(wrapping),
                    })
                }
            };
        }
    }
}

impl TryFrom<crate::query::InputValue> for InputValue {
    type Error = SchemaError;

    fn try_from(value: crate::query::InputValue) -> Result<Self, Self::Error> {
        Ok(InputValue {
            name: value.name,
            description: value.description,
            ty: value.ty.try_into()?,
            default_value: value.default_value,
        })
    }
}

impl From<crate::query::EnumValue> for EnumValue {
    fn from(value: crate::query::EnumValue) -> Self {
        EnumValue {
            name: value.name,
            description: value.description,
            deprecated: Deprecated::new(value.is_deprecated, value.deprecation_reason),
        }
    }
}

impl TryFrom<crate::query::Directive> for Directive {
    type Error = SchemaError;

    fn try_from(value: crate::query::Directive) -> Result<Self, Self::Error> {
        Ok(Directive {
            name: value.name,
            description: value.description,
            args: value
                .args
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            locations: value.locations,
        })
    }
}

impl crate::query::NamedType {
    fn into_name(self) -> Result<String, SchemaError> {
        self.name.ok_or(SchemaError::TypeMissingName)
    }
}
