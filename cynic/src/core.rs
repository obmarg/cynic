use crate::{queries::SelectionBuilder, QueryVariables};

/// Indicates that a type may be used as part of a graphql query.
///
/// This will usually be derived, but can be manually implemented if required.
pub trait QueryFragment<'de>: serde::Deserialize<'de> {
    /// The type in a schema that this `QueryFragment` represents
    type SchemaType;

    /// The variables that are required to execute this `QueryFragment`
    type Variables: QueryVariables;

    /// The name of the type in the GraphQL schema
    const TYPE: Option<&'static str> = None;

    /// Adds this fragment to the query being built by `builder`
    fn query(builder: SelectionBuilder<Self::SchemaType, Self::Variables>);
}

impl<'de, T> QueryFragment<'de> for Option<T>
where
    T: QueryFragment<'de>,
{
    type SchemaType = Option<T::SchemaType>;
    type Variables = T::Variables;

    fn query(builder: SelectionBuilder<Self::SchemaType, Self::Variables>) {
        T::query(builder.into_inner())
    }
}

impl<'de, T> QueryFragment<'de> for Vec<T>
where
    T: QueryFragment<'de>,
{
    type SchemaType = Vec<T::SchemaType>;
    type Variables = T::Variables;

    fn query(builder: SelectionBuilder<Self::SchemaType, Self::Variables>) {
        T::query(builder.into_inner())
    }
}

impl<'de, T> QueryFragment<'de> for Box<T>
where
    T: QueryFragment<'de>,
{
    type SchemaType = T::SchemaType;
    type Variables = T::Variables;

    fn query(builder: SelectionBuilder<Self::SchemaType, Self::Variables>) {
        T::query(builder)
    }
}

impl<'de, T> QueryFragment<'de> for std::rc::Rc<T>
where
    Self: serde::Deserialize<'de>,
    T: QueryFragment<'de>,
{
    type SchemaType = T::SchemaType;
    type Variables = T::Variables;

    fn query(builder: SelectionBuilder<Self::SchemaType, Self::Variables>) {
        T::query(builder)
    }
}

impl<'de, T> QueryFragment<'de> for std::sync::Arc<T>
where
    Self: serde::Deserialize<'de>,
    T: QueryFragment<'de>,
{
    type SchemaType = T::SchemaType;
    type Variables = T::Variables;

    fn query(builder: SelectionBuilder<Self::SchemaType, Self::Variables>) {
        T::query(builder)
    }
}

impl<'de> QueryFragment<'de> for bool {
    type SchemaType = bool;
    type Variables = ();

    fn query(_builder: SelectionBuilder<Self::SchemaType, Self::Variables>) {}
}

impl<'de> QueryFragment<'de> for String {
    type SchemaType = String;
    type Variables = ();

    fn query(_builder: SelectionBuilder<Self::SchemaType, Self::Variables>) {}
}

/// A QueryFragment that contains a set of inline fragments
///
/// This should be derived on an enum with newtype variants where each
/// inner type is a `QueryFragment` for an apppropriate type.
pub trait InlineFragments<'de>: QueryFragment<'de> {
    /// Attempts to deserialize a variant with the given typename.
    fn deserialize_variant<D>(typename: &str, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>;
}

/// A GraphQL Enum.
///
/// Note that in GraphQL these can't contain data - they are just a set of strings.
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

impl<T> InputObject for Vec<T>
where
    T: InputObject,
{
    type SchemaType = Vec<T::SchemaType>;
}

impl<T> InputObject for Box<T>
where
    T: InputObject,
{
    type SchemaType = T::SchemaType;
}
