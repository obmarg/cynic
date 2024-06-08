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

use std::borrow::{Borrow, Cow};

use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serializer};

use crate::__private::{ScalarDeserialize, ScalarSerialize};

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

// TODO: Think about the names of the scalar traits....

/// Indicates that a type is a scalar that maps to the given schema scalar.
///
/// Note that this type is actually implemented on the users types.
pub trait IsScalar<SchemaType> {
    /// The schema marker type this scalar represents.
    type SchemaType;

    // TODO: serialize should maybe be on an OutputScalar trait
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

// TODO: serialize should maybe be on an InputScalar trait
// or maybe just ScalarSerialize/ScalarDeserialize?  not sure...
pub trait IsOutputScalar<SchemaType>: Sized {
    /// The schema marker type this scalar represents.
    type SchemaType;

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
        <U as IsScalar<T>>::serialize(self, serializer)
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
        match self {
            Some(inner) => inner.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}

impl<T, U> IsOutputScalar<Option<T>> for Option<U>
where
    U: IsOutputScalar<T>,
{
    type SchemaType = Option<U::SchemaType>;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(
            Option::<ScalarDeserialize<U, T>>::deserialize(deserializer)?
                .map(ScalarDeserialize::into_inner),
        )
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
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for item in self {
            seq.serialize_element(&ScalarSerialize::new(item))?;
        }
        seq.end()
    }
}

impl<T, U> IsOutputScalar<Vec<T>> for Vec<U>
where
    U: IsOutputScalar<T>,
{
    type SchemaType = Vec<U::SchemaType>;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Vec::<ScalarDeserialize<U, T>>::deserialize(deserializer)?
            .into_iter()
            .map(ScalarDeserialize::into_inner)
            .collect())
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
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for item in self {
            seq.serialize_element(&ScalarSerialize::new(item))?;
        }
        seq.end()
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
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for item in self {
            seq.serialize_element(&ScalarSerialize::new(item))?;
        }
        seq.end()
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
        self.as_ref().serialize(serializer)
    }
}

impl<T, U: ?Sized> IsOutputScalar<Box<T>> for Box<U>
where
    U: IsOutputScalar<T>,
{
    type SchemaType = Box<U::SchemaType>;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        U::deserialize(deserializer).map(Box::new)
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
        self.as_ref().serialize(serializer)
    }
}

impl<T, U: ?Sized> IsOutputScalar<T> for std::borrow::Cow<'_, U>
where
    U: IsOutputScalar<T> + ToOwned,
{
    type SchemaType = U::SchemaType;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Cow::Owned(U::deserialize(deserializer)?.to_owned()))
    }
}

impl IsOutputScalar<String> for std::borrow::Cow<'static, str> {
    type SchemaType = String;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Cow::Owned(
            <String as serde::Deserialize>::deserialize(deserializer)?.to_owned(),
        ))
    }
}

impl IsScalar<bool> for bool {
    type SchemaType = bool;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl IsOutputScalar<bool> for bool {
    type SchemaType = bool;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
    }
}

impl IsScalar<String> for String {
    type SchemaType = String;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl IsOutputScalar<String> for String {
    type SchemaType = String;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
    }
}

impl IsScalar<String> for str {
    type SchemaType = String;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl IsScalar<i32> for i32 {
    type SchemaType = i32;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl IsOutputScalar<i32> for i32 {
    type SchemaType = i32;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
    }
}

impl IsScalar<f64> for f64 {
    type SchemaType = f64;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl IsOutputScalar<f64> for f64 {
    type SchemaType = f64;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
    }
}

impl IsScalar<crate::Id> for crate::Id {
    type SchemaType = crate::Id;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl IsOutputScalar<crate::Id> for crate::Id {
    type SchemaType = crate::Id;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
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
