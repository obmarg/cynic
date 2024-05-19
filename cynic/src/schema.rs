//! Traits for describing a GraphQL schema in Rust.
//!
//! The `use_schema` macro will mostly output types that make use of the
//! traits in this module to describe the schema.  The derives then combine
//! these types with the `QueryBuilder` type to enforce the restrictions
//! of the schema.
//!
//! Note that this module is mostly concerned with the marker types output
//! by `use_schema`, _not_ the actual types users work with.  The traits
//! will usually be implemented on marker types, the geneirc parameters will
//! usually be marker types and the associated types will also usually be
//! markers.

use serde::{Deserializer, Serializer};

/// Indicates that a struct represents a Field in a graphql schema.
pub trait Field {
    /// The schema marker type of this field.
    type Type;

    /// the name of this field
    const NAME: &'static str;
}

// TODO: Get the terminology straight in this file, it's a mess.

/// Indicates that a type has a given field
///
/// This should be implemented several times for any given type,
/// once per field. `FieldMarker` should be the marker type for
/// the field,
pub trait HasField<FieldMarker> {
    /// The schema marker type of this field.
    type Type;
}

/// Indicates that an input object has a given field
///
/// This should be implemented several times for any given input object,
/// once per field. `FieldMarker` should be the marker type for the field,
/// and `FieldType` should be the schema marker type of the field.
pub trait HasInputField<FieldMarker, FieldType> {}

/// Indicates that a field has an argument
///
/// This should be implemented on the field marker type for each argument
/// that field has.  `ArgumentMarker` should be the marker type for the
/// argument.
pub trait HasArgument<ArgumentMarker> {
    /// The schema marker type of this argument.
    type ArgumentType;

    /// The name of this argument
    const NAME: &'static str;
}

/// Indicates that a type is a scalar that maps to the given schema scalar.
///
/// Note that this type is actually implemented on the users types.
pub trait IsScalar<SchemaType> {
    /// The schema marker type this scalar represents.
    type SchemaType;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl<T, U: ?Sized> IsScalar<T> for &U
where
    U: IsScalar<T>,
{
    type SchemaType = U::SchemaType;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <U as IsScalar<U>>::serialize(self, serializer)
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <U as IsScalar<U>>::deserialize(deserializer)
    }
}

impl<T, U> IsScalar<Option<T>> for Option<U>
where
    U: IsScalar<T>,
{
    type SchemaType = Option<U::SchemaType>;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl<T, U> IsScalar<Vec<T>> for Vec<U>
where
    U: IsScalar<T>,
{
    type SchemaType = Vec<U::SchemaType>;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl<T, U> IsScalar<Vec<T>> for [U]
where
    U: IsScalar<T>,
{
    type SchemaType = Vec<U::SchemaType>;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl<T, U, const SIZE: usize> IsScalar<Vec<T>> for [U; SIZE]
where
    U: IsScalar<T>,
{
    type SchemaType = Vec<U::SchemaType>;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl<T, U: ?Sized> IsScalar<Box<T>> for Box<U>
where
    U: IsScalar<T>,
{
    type SchemaType = Box<U::SchemaType>;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl<T, U: ?Sized> IsScalar<T> for std::borrow::Cow<'_, U>
where
    U: IsScalar<T> + ToOwned,
{
    type SchemaType = U::SchemaType;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl IsScalar<bool> for bool {
    type SchemaType = bool;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl IsScalar<String> for String {
    type SchemaType = String;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl IsScalar<String> for str {
    type SchemaType = String;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl IsScalar<i32> for i32 {
    type SchemaType = i32;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl IsScalar<f64> for f64 {
    type SchemaType = f64;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

impl IsScalar<crate::Id> for crate::Id {
    type SchemaType = crate::Id;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

/// A marker trait that indicates a particular type is at the root of a GraphQL
/// schemas query hierarchy.
pub trait QueryRoot {}

/// A marker trait that indicates a particular type is at the root of a GraphQL
/// schemas mutation hierarchy.
pub trait MutationRoot {}

/// A marker trait that indicates a particular type is at the root of a GraphQL
/// schemas subscription hierarchy.
pub trait SubscriptionRoot {}

/// Indicates that a type has a subtype relationship with another type
pub trait HasSubtype<Type> {}

/// A marker type with a name.
pub trait NamedType {
    /// The name of this type
    const NAME: &'static str;
}

impl NamedType for i32 {
    const NAME: &'static str = "Int";
}

impl NamedType for f64 {
    const NAME: &'static str = "Float";
}

impl NamedType for String {
    const NAME: &'static str = "String";
}

impl NamedType for bool {
    const NAME: &'static str = "Boolean";
}

impl NamedType for crate::Id {
    const NAME: &'static str = "ID";
}

/// Indicates that a type is an `InputObject`
pub trait InputObjectMarker {}

/// Indicates that a type represents a GraphQL directive that can be used
/// in field position.
pub trait FieldDirective {
    /// The name of the directive in GraphQL
    const NAME: &'static str;
}
