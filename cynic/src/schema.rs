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

use std::borrow::Cow;

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

/// Indicates that a type can be used as a graphql scalar in input position
///
/// This should be implemented for any scalar types that need to be used as arguments
/// or appear on fields of input objects.
///
/// The SchemaType generic parameter should be set to a marker type from the users schema module -
/// this indicates which scalar(s) this type represents in a graphql schema.
#[diagnostic::on_unimplemented(
    message = "{Self} cannot be used for fields of type {SchemaType}",
    label = "The GraphQL schema expects a {SchemaType} here but {Self} is not registered for use with fields of that type",
    note = "You either need to fix the type used on this field, or register {Self} for use as a {SchemaType}"
)]
pub trait IsScalar<SchemaType> {
    /// Serializes Self using the provided Serializer
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

// TODO: serialize should maybe be on an InputScalar trait
// or maybe just ScalarSerialize/ScalarDeserialize?  not sure...

/// Indicates that a type can be used as a graphql scalar in output position
///
/// This should be implemented for any scalars that are used as fields in types implementing
/// QueryFragment.
///
/// The SchemaType generic parameter should be set to a marker type from the users schema module -
/// this indicates which scalar(s) this type represents in a graphql schema.
#[diagnostic::on_unimplemented(
    message = "{Self} cannot be used for fields of type {SchemaType}",
    label = "The GraphQL schema expects a {SchemaType} here but {Self} is not registered for use with fields of that type",
    note = "You either need to fix the type used on this field, or register {Self} for use as a {SchemaType}"
)]
pub trait IsOutputScalar<'de, SchemaType>: Sized {
    /// Deserializes Self using the provided Deserializer
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl<T, U: ?Sized> IsScalar<T> for &U
where
    U: IsScalar<T>,
{
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

impl<'de, T, U> IsOutputScalar<'de, Option<T>> for Option<U>
where
    U: IsOutputScalar<'de, T>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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

impl<'de, T, U> IsOutputScalar<'de, Vec<T>> for Vec<U>
where
    U: IsOutputScalar<'de, T>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

impl<'de, T, U: ?Sized> IsOutputScalar<'de, Box<T>> for Box<U>
where
    U: IsOutputScalar<'de, T>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

impl<'de, T, U: ?Sized> IsOutputScalar<'de, T> for std::borrow::Cow<'_, U>
where
    U: IsOutputScalar<'de, T> + ToOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Cow::Owned(U::deserialize(deserializer)?.to_owned()))
    }
}

impl<'de> IsOutputScalar<'de, String> for std::borrow::Cow<'static, str> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Cow::Owned(
            <String as serde::Deserialize>::deserialize(deserializer)?.to_owned(),
        ))
    }
}

impl IsScalar<bool> for bool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl<'de> IsOutputScalar<'de, bool> for bool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
    }
}

impl IsScalar<String> for String {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl<'de> IsOutputScalar<'de, String> for String {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
    }
}

impl IsScalar<String> for str {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl IsScalar<i32> for i32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl<'de> IsOutputScalar<'de, i32> for i32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
    }
}

impl IsScalar<f64> for f64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl<'de> IsOutputScalar<'de, f64> for f64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
    }
}

impl IsScalar<crate::Id> for crate::Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(self, serializer)
    }
}

impl<'de> IsOutputScalar<'de, crate::Id> for crate::Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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
