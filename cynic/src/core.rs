use std::borrow::Cow;

use crate::{queries::SelectionBuilder, MaybeUndefined, QueryVariablesFields};

/// A trait that marks a type as part of a GraphQL query.
///
/// This will usually be derived, but can be manually implemented if required.
pub trait QueryFragment: Sized {
    /// The type in a schema that this `QueryFragment` represents
    type SchemaType;

    /// The variables that are required to execute this `QueryFragment`
    type VariablesFields: QueryVariablesFields;

    /// The name of the type in the GraphQL schema
    const TYPE: Option<&'static str> = None;

    /// Adds this fragment to the query being built by `builder`
    fn query(builder: SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>);

    /// The name of this fragment, useful for operations, maybe fragments if we ever support them...
    fn name() -> Option<Cow<'static, str>> {
        // Most QueryFragments don't need a name so return None
        None
    }
}

impl<T> QueryFragment for Option<T>
where
    T: QueryFragment,
{
    type SchemaType = Option<T::SchemaType>;
    type VariablesFields = T::VariablesFields;

    fn query(builder: SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>) {
        T::query(builder.into_inner())
    }
}

impl<T> QueryFragment for MaybeUndefined<T>
where
    T: QueryFragment,
{
    type SchemaType = Option<T::SchemaType>;
    type VariablesFields = T::VariablesFields;

    fn query(builder: SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>) {
        T::query(builder.into_inner())
    }
}

impl<T> QueryFragment for Vec<T>
where
    T: QueryFragment,
{
    type SchemaType = Vec<T::SchemaType>;
    type VariablesFields = T::VariablesFields;

    fn query(builder: SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>) {
        T::query(builder.into_inner())
    }
}

impl<T> QueryFragment for Box<T>
where
    T: QueryFragment,
{
    type SchemaType = T::SchemaType;
    type VariablesFields = T::VariablesFields;

    const TYPE: Option<&'static str> = T::TYPE;

    fn query(builder: SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>) {
        T::query(builder)
    }
}

impl<T> QueryFragment for std::rc::Rc<T>
where
    T: QueryFragment,
{
    type SchemaType = T::SchemaType;
    type VariablesFields = T::VariablesFields;

    const TYPE: Option<&'static str> = T::TYPE;

    fn query(builder: SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>) {
        T::query(builder)
    }
}

impl<T> QueryFragment for std::sync::Arc<T>
where
    T: QueryFragment,
{
    type SchemaType = T::SchemaType;
    type VariablesFields = T::VariablesFields;

    const TYPE: Option<&'static str> = T::TYPE;

    fn query(builder: SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>) {
        T::query(builder)
    }
}

impl QueryFragment for bool {
    type SchemaType = bool;
    type VariablesFields = ();

    fn query(_builder: SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>) {}
}

impl QueryFragment for String {
    type SchemaType = String;
    type VariablesFields = ();

    fn query(_builder: SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>) {}
}

/// A QueryFragment that contains a set of inline fragments
///
/// This should be derived on an enum with newtype variants where each
/// inner type is a `QueryFragment` for an appropriate type.
pub trait InlineFragments<'de>: QueryFragment + serde::de::Deserialize<'de> {
    /// Attempts to deserialize a variant with the given typename.
    fn deserialize_variant<D>(typename: &str, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>;
}

/// A GraphQL Enum.
///
/// Note that in GraphQL these can't contain data - they are just a set of
/// strings.
///
/// This should be be derived on an enum with unit variants.
pub trait Enum: serde::de::DeserializeOwned + serde::Serialize {
    /// The enum in the schema that this type represents.
    type SchemaType;
}

impl<T> Enum for Option<T>
where
    T: Enum,
{
    type SchemaType = Option<T::SchemaType>;
}

impl<T> Enum for Vec<T>
where
    T: Enum,
{
    type SchemaType = Vec<T::SchemaType>;
}

impl<T> Enum for Box<T>
where
    T: Enum,
{
    type SchemaType = T::SchemaType;
}

/// A GraphQL input object.
///
/// This should be derived on a struct.
pub trait InputObject: serde::Serialize {
    /// The input object in the schema that this type represents.
    type SchemaType;
}

impl<T> InputObject for Option<T>
where
    T: InputObject,
{
    type SchemaType = Option<T::SchemaType>;
}

impl<T> InputObject for MaybeUndefined<T>
where
    T: InputObject,
{
    type SchemaType = Option<T::SchemaType>;
}

impl<T> InputObject for Vec<T>
where
    T: InputObject,
{
    type SchemaType = Vec<T::SchemaType>;
}

impl<T> InputObject for [T]
where
    T: InputObject,
{
    type SchemaType = Vec<T::SchemaType>;
}

impl<T, const SIZE: usize> InputObject for [T; SIZE]
where
    T: InputObject,
    Self: serde::Serialize,
{
    type SchemaType = Vec<T::SchemaType>;
}

impl<T> InputObject for Box<T>
where
    T: InputObject,
{
    type SchemaType = T::SchemaType;
}

impl<T: ?Sized> InputObject for &T
where
    T: InputObject,
{
    type SchemaType = T::SchemaType;
}
