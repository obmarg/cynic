#![allow(dead_code, unused_variables, missing_docs)]
// TODO: Don't allow the above

use std::fmt::Write;

// TODO: Everything in here is actually typed.  Need an untyped core with this
// layered on top...

use std::marker::PhantomData;

use crate::{indent::indented, queries::QueryBuilder, schema};

// Annoyingly this means people can't derive Deserialize _as well as_ use cynics derives.
// But whatever, don't do that people?  I _think_ it's an OK limitation.
pub trait QueryFragment<'de>: serde::Deserialize<'de> {
    type SchemaType;

    fn query(builder: QueryBuilder<Self::SchemaType>);
}

impl<'de, T> QueryFragment<'de> for Option<T>
where
    T: QueryFragment<'de>,
{
    type SchemaType = Option<T::SchemaType>;

    fn query(builder: QueryBuilder<Self::SchemaType>) {
        T::query(builder.into_inner())
    }
}

impl<'de, T> QueryFragment<'de> for Vec<T>
where
    T: QueryFragment<'de>,
{
    type SchemaType = Vec<T::SchemaType>;

    fn query(builder: QueryBuilder<Self::SchemaType>) {
        T::query(builder.into_inner())
    }
}

impl<'de, T> QueryFragment<'de> for Box<T>
where
    T: QueryFragment<'de>,
{
    type SchemaType = T::SchemaType;

    fn query(builder: QueryBuilder<Self::SchemaType>) {
        T::query(builder)
    }
}

impl<'de> QueryFragment<'de> for bool {
    type SchemaType = bool;

    fn query(builder: QueryBuilder<Self::SchemaType>) {}
}

// TODO: Can I also impl this for &'static str?
impl<'de> QueryFragment<'de> for String {
    type SchemaType = String;

    fn query(builder: QueryBuilder<Self::SchemaType>) {}
}

// TODO: Does this need a TypeLock on it?
pub trait Enum<'de>: serde::Deserialize<'de> + serde::Serialize {}

// TODO: Does this need a TypeLock on it?
pub trait Scalar<'de>: serde::Deserialize<'de> + serde::Serialize {}

pub trait Variable {
    type ArgumentStruct;
    type SchemaType;

    fn name() -> &'static str;
    // TODO: Do we need a name func in here.  Probably.
}

// TODO: Might want recursive impls of Variable for Vec & Option?
// Such that a T is valid for an Option<T> variable or a Vec<T>
