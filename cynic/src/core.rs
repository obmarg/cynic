#![allow(dead_code, unused_variables, missing_docs)]
// TODO: Don't allow the above

use std::fmt::Write;

// TODO: Everything in here is actually typed.  Need an untyped core with this
// layered on top...

use std::marker::PhantomData;

use serde::Deserialize;

use crate::{indent::indented, queries::QueryBuilder, schema};

// Annoyingly this means people can't derive Deserialize _as well as_ use cynics derives.
// But whatever, don't do that people?  I _think_ it's an OK limitation.
pub trait QueryFragment<'de>: serde::Deserialize<'de> {
    type SchemaType;
    type Variables;

    fn query(builder: QueryBuilder<Self::SchemaType>);
}

impl<'de, T> QueryFragment<'de> for Option<T>
where
    T: QueryFragment<'de>,
{
    type SchemaType = Option<T::SchemaType>;
    type Variables = T::Variables;

    fn query(builder: QueryBuilder<Self::SchemaType>) {
        T::query(builder.into_inner())
    }
}

impl<'de, T> QueryFragment<'de> for Vec<T>
where
    T: QueryFragment<'de>,
{
    type SchemaType = Vec<T::SchemaType>;
    type Variables = T::Variables;

    fn query(builder: QueryBuilder<Self::SchemaType>) {
        T::query(builder.into_inner())
    }
}

impl<'de, T> QueryFragment<'de> for Box<T>
where
    T: QueryFragment<'de>,
{
    type SchemaType = T::SchemaType;
    type Variables = T::Variables;

    fn query(builder: QueryBuilder<Self::SchemaType>) {
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

    fn query(builder: QueryBuilder<Self::SchemaType>) {
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

    fn query(builder: QueryBuilder<Self::SchemaType>) {
        T::query(builder)
    }
}

impl<'de> QueryFragment<'de> for bool {
    type SchemaType = bool;
    type Variables = ();

    fn query(builder: QueryBuilder<Self::SchemaType>) {}
}

// TODO: Can I also impl this for &'static str?
impl<'de> QueryFragment<'de> for String {
    type SchemaType = String;
    type Variables = ();

    fn query(builder: QueryBuilder<Self::SchemaType>) {}
}

// TODO: Does this need a TypeLock on it?
pub trait Enum<'de>: serde::Deserialize<'de> + serde::Serialize {}

// TODO: Does this need a TypeLock on it?
pub trait Scalar<'de>: serde::Deserialize<'de> + serde::Serialize {}

pub trait InputObject: serde::Serialize {
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

pub trait QueryVariables {
    type Fields;
}

// TODO: Think about this name & where we should put it
pub struct VariableDefinition<ArgumentStruct, Type> {
    pub name: &'static str,
    phantom: PhantomData<fn() -> (ArgumentStruct, Type)>,
}

impl<ArgumentStruct, Type> VariableDefinition<ArgumentStruct, Type> {
    pub fn new(name: &'static str) -> Self {
        VariableDefinition {
            name,
            phantom: PhantomData,
        }
    }
}

// TODO: Might want recursive impls of Variable for Vec & Option?
// Such that a T is valid for an Option<T> variable or a Vec<T>
