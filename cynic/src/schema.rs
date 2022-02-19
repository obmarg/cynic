#![allow(dead_code, unused_variables, missing_docs)]
// TODO: Don't allow the above

pub trait Field {
    type SchemaType;

    fn name() -> &'static str;
}

// TODO: Get the terminology straight in this file, it's a mess.

pub trait HasField<FieldMarker> {
    type Type;
}

pub trait HasInputField<FieldMarker, FieldType> {}

pub trait HasArgument<ArgumentName> {
    type ArgumentSchemaType;

    // TODO: Maybe move the name to that named trait def?
    fn name() -> &'static str;
}

pub trait IsScalar<SchemaType> {
    type SchemaType;
}

impl<T, U> IsScalar<Option<T>> for Option<U>
where
    U: IsScalar<T>,
{
    type SchemaType = Option<U::SchemaType>;
}

impl<T, U> IsScalar<Vec<T>> for Vec<U>
where
    U: IsScalar<T>,
{
    type SchemaType = Vec<U::SchemaType>;
}

impl<T, U> IsScalar<Box<T>> for Box<U>
where
    U: IsScalar<T>,
{
    type SchemaType = Box<U::SchemaType>;
}

impl IsScalar<bool> for bool {
    type SchemaType = bool;
}

impl IsScalar<String> for String {
    type SchemaType = String;
}

impl IsScalar<i32> for i32 {
    type SchemaType = i32;
}

impl IsScalar<f64> for f64 {
    type SchemaType = f64;
}

impl IsScalar<crate::Id> for crate::Id {
    type SchemaType = crate::Id;
}

/// A marker trait that indicates a particular type is at the root of a GraphQL schemas query
/// hierarchy.
pub trait QueryRoot {}

/// A marker trait that indicates a particular type is at the root of a GraphQL schemas
/// mutation hierarchy.
pub trait MutationRoot {}

/// A marker trait that indicates a particular type is at the root of a GraphQL schemas
/// subscription hierarchy.
pub trait SubscriptionRoot {}

pub trait HasSubtype<Type> {}

pub trait NamedType {
    fn name() -> &'static str;
}

pub trait InputObjectMarker {}
