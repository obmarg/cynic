use crate::query::DirectiveLocation;

impl crate::query::IntrospectionQuery {
    /// Converts the results of an IntrospectionQuery into a `Schema`,
    /// which has some stronger types than those offered by the introspection query
    pub fn into_schema(self) -> Result<Schema, SchemaError> {
        Schema::try_from(self.introspected_schema)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// A GraphQL schema
pub struct Schema {
    /// The name of the `query` root operation type
    pub query_type: String,
    /// The name of the `mutation` root operation type
    pub mutation_type: Option<String>,
    /// The name of the `subscription` root operation type
    pub subscription_type: Option<String>,
    /// All the `Type`s available in the schema
    pub types: Vec<Type>,
    /// All the `Directive`s available in the schema
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// A directive either used in the schema or available to queries
pub struct Directive {
    /// The name of the directive
    pub name: String,
    /// A description of the directive
    pub description: Option<String>,
    /// Any arguments that can be provided to the directive
    pub args: Vec<InputValue>,
    /// The locations where the directive may be used
    pub locations: Vec<DirectiveLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A type defined in a `Schema`
pub enum Type {
    /// The type is an `ObjectType`
    Object(ObjectType),
    /// The type is an `InputObjectType`
    InputObject(InputObjectType),
    /// The type is an `EnumType`
    Enum(EnumType),
    /// The type is an `InterfaceType`
    Interface(InterfaceType),
    /// The type is a `UnionType`
    Union(UnionType),
    /// The type is a `Scalar`
    Scalar(ScalarType),
}

impl Type {
    /// The name of the type
    pub fn name(&self) -> &str {
        match self {
            Type::Object(inner) => &inner.name,
            Type::InputObject(inner) => &inner.name,
            Type::Enum(inner) => &inner.name,
            Type::Interface(inner) => &inner.name,
            Type::Union(inner) => &inner.name,
            Type::Scalar(inner) => &inner.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// GraphQL Objects represent a list of named fields, each of which
/// yield a value of a specific type.
pub struct ObjectType {
    /// The name of the object
    pub name: String,
    /// A description of the object
    pub description: Option<String>,
    /// The fields of the object
    pub fields: Vec<Field>,
    /// Any interfaces this object implements
    pub interfaces: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// A GraphQL Input Object defines a set of input fields; the input fields are
/// either scalars, enums, or other input objects
pub struct InputObjectType {
    /// The name of the input object
    pub name: String,
    /// A description of the input object
    pub description: Option<String>,
    /// The fields of the input object
    pub fields: Vec<InputValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// GraphQL Enum types, like Scalar types, also represent leaf values in a GraphQL
/// type system. However Enum types describe the set of possible values.
pub struct EnumType {
    /// The name of the enum
    pub name: String,
    /// A description of the enum
    pub description: Option<String>,
    /// The possible values this enum can have
    pub values: Vec<EnumValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// Interfaces are an abstract type where there are common fields declared.
pub struct InterfaceType {
    /// The name of the interface
    pub name: String,
    /// A description of the interface
    pub description: Option<String>,
    /// The fields of the interface
    pub fields: Vec<Field>,
    /// Any interfaces this interface also implements
    pub interfaces: Vec<String>,
    /// The set of types that implement this interface
    pub possible_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// GraphQL Unions represent an object that could be one of a list of GraphQL
/// Object types, but provides for no guaranteed fields between those types
pub struct UnionType {
    /// The name of the union
    pub name: String,
    /// A description of the union
    pub description: Option<String>,
    /// The set of types that this interface could be
    pub possible_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// Scalar types represent primitive leaf values in a GraphQL type system.
///
/// GraphQL provides a number of built-in scalars, however type systems may also
/// add additional custom scalars to introduce additional semantic meaning.
pub struct ScalarType {
    /// The name of the scalar
    pub name: String,
    /// A description of the scalar
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// One of the possible values of an [EnumType]
pub struct EnumValue {
    /// The name of the enum
    pub name: String,
    /// A description of the value
    pub description: Option<String>,
    /// Whether this value is deprecated and should no longer be used.
    pub deprecated: Deprecated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Whether a [Field] or [EnumValue] is deprecated.
pub enum Deprecated {
    /// The [Field] or [EnumValue] is not deprecated.
    No,
    /// The [Field] or [EnumValue] is deprecated, with an optional reason
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
/// One of the fields of an [ObjectType] or [InterfaceType]
pub struct Field {
    /// The name of the field
    pub name: String,
    /// A description of the field
    pub description: Option<String>,
    /// A list of arguments this field accepts.
    pub args: Vec<InputValue>,
    /// The type of value returned by this field
    pub ty: FieldType,
    /// Whether this field is deprecated and should no longer be used.
    pub deprecated: Deprecated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// Represents field and directive arguments as well as the fields of an input object.
pub struct InputValue {
    /// The name of the argument or field
    pub name: String,
    /// A description of the argument or field
    pub description: Option<String>,
    /// The type of this argument or field
    pub ty: FieldType,
    /// An optional default value for this field, represented as a GraphQL literal
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// The type of a [Field] or [InputValue]
///
/// Provides a [std::fmt::Display] impl that can be used to get the GraphQL string
/// representation of this [FieldType].
pub struct FieldType {
    /// The "wrapping types" for a field - whether it is wrapped in NonNull or a List
    pub wrapping: FieldWrapping,
    /// The named type contained within any `FieldWrapping`
    pub name: String,
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FieldType { wrapping, name } = self;
        let wrapping_types = wrapping.into_iter().collect::<Vec<_>>();
        for wrapping_type in &wrapping_types {
            match wrapping_type {
                WrappingType::List => write!(f, "[")?,
                WrappingType::NonNull => {}
            }
        }
        write!(f, "{name}")?;
        for wrapping_type in wrapping_types.iter().rev() {
            match wrapping_type {
                WrappingType::List => write!(f, "]")?,
                WrappingType::NonNull => write!(f, "!")?,
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// A list of [WrappingType]s that wrap a [FieldType]
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
/// Types that can wrap either named types or other wrapper types
pub enum WrappingType {
    /// Wraps a named or wrapper type in a list
    ///
    /// This type can be nested to form lists, or combined with [WrappingType::NonNull]
    /// to define non nullable lists.
    List,
    /// Marks the wrapped type as non nullable.
    NonNull,
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
/// An error that can occur when building the schema
pub enum SchemaError {
    /// A type in the introspection output was missing a name
    #[error("A type in the introspection output was missing a name")]
    TypeMissingName,
    /// Found a list wrapper type in a position that should contain a named type
    #[error("Found a list wrapper type in a position that should contain a named type")]
    UnexpectedList,
    /// Found a non-null wrapper type in a position that should contain a named type
    #[error("Found a non-null wrapper type in a position that should contain a named type")]
    UnexpectedNonNull,
    /// Found a wrapping type that had no inner ofType
    #[error("Found a wrapping type that had no inner ofType")]
    WrappingTypeWithNoInner,
    /// Found a wrapping type that was too nested
    #[error("Found a wrapping type that was too nested")]
    TooMuchWrapping,
}

impl TryFrom<crate::query::IntrospectedSchema> for Schema {
    type Error = SchemaError;

    fn try_from(schema: crate::query::IntrospectedSchema) -> Result<Self, Self::Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query;

    #[test]
    fn test_field_type_to_string() {
        let ty = FieldType::try_from(query::FieldType {
            kind: query::TypeKind::NonNull,
            name: None,
            of_type: Some(Box::new(query::FieldType {
                kind: query::TypeKind::List,
                name: None,
                of_type: Some(Box::new(query::FieldType {
                    kind: query::TypeKind::Scalar,
                    name: Some("Int".into()),
                    of_type: None,
                })),
            })),
        })
        .unwrap();

        assert_eq!(ty.to_string(), "[Int]!");

        let ty = FieldType::try_from(query::FieldType {
            kind: query::TypeKind::Object,
            name: Some("MyObject".into()),
            of_type: None,
        })
        .unwrap();

        assert_eq!(ty.to_string(), "MyObject");
    }
}
