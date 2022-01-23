#![allow(dead_code, unused_variables, missing_docs)]
// TODO: Don't allow the above

pub trait Field {
    type SchemaType;

    fn name() -> &'static str;
}

// TODO: Get the terminology straight in this file, it's a mess.

pub trait HasField<FieldMarker, FieldType> {}

pub trait HasInputField<FieldMarker, FieldType> {
    type ArgumentKind;
}

pub trait HasArgument<ArgumentName> {
    type ArgumentSchemaType;

    // TODO: Constrain this to the InputKind trait below?
    type ArgumentKind;

    // TODO: Maybe move the name to that named trait def?
    fn name() -> &'static str;
}

// TODO: Make this sealed, make the things below impl it.
trait ArgumentKind {}

pub struct InputObjectArgument;
pub struct EnumArgument;
pub struct ScalarArgument;

// impl<T, U> HasArgument<Option<T>> for Option<U>
// where
//     U: HasArgument<T>,
// {
//     type ArgumentSchemaType = Option<U::ArgumentSchemaType>;

//     fn name() -> &'static str {
//         todo!()
//     }
// }

// impl<T, U> HasArgument<Vec<T>> for Vec<U>
// where
//     U: HasArgument<T>,
// {
//     type ArgumentSchemaType = Vec<U::ArgumentSchemaType>;

//     fn name() -> &'static str {
//         todo!()
//     }
// }

// impl<T, U> HasArgument<Box<T>> for Box<U>
// where
//     U: HasArgument<T>,
// {
//     type ArgumentSchemaType = Box<U::ArgumentSchemaType>;

//     fn name() -> &'static str {
//         todo!()
//     }
// }

// TODO: Name of this vs the actual Value type I want to output
pub trait InputValue<SchemaType> {
    // TODO: Bet the self type & references are going to be a PITA with this...
    // fn to_actual_value(&self) -> ();
}

impl InputValue<crate::Id> for crate::Id {}
impl InputValue<Option<crate::Id>> for crate::Id {}
impl InputValue<Option<crate::Id>> for Option<crate::Id> {}
impl InputValue<Vec<crate::Id>> for crate::Id {}
impl InputValue<Vec<crate::Id>> for Vec<crate::Id> {}

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

pub trait IsEnum<SchemaType> {
    type SchemaType;
}

impl<T, U> IsEnum<Option<T>> for Option<U>
where
    U: IsEnum<T>,
{
    type SchemaType = Option<U::SchemaType>;
}

impl<T, U> IsEnum<Vec<T>> for Vec<U>
where
    U: IsEnum<T>,
{
    type SchemaType = Vec<U::SchemaType>;
}

impl<T, U> IsEnum<Box<T>> for Box<U>
where
    U: IsEnum<T>,
{
    type SchemaType = Box<U::SchemaType>;
}

pub trait IsInputObject<SchemaType> {}

impl<T, U> IsInputObject<Option<T>> for Option<U> where U: IsInputObject<T> {}

impl<T, U> IsInputObject<Vec<T>> for Vec<U> where U: IsInputObject<T> {}

impl<T, U> IsInputObject<Box<T>> for Box<U> where U: IsInputObject<T> {}

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
