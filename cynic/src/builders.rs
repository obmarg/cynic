use crate::operation::StreamingOperation;

use super::{Operation, QueryFragment};

/// Provides a `build` function on `QueryFragment`s that represent a query
pub trait QueryBuilder<'de>: Sized {
    /// The type that this query takes as variables.
    /// May be `()` if no variables are accepted.
    type Variables;

    /// Constructs a query operation for this QueryFragment.
    fn build(vars: Self::Variables) -> Operation<Self, Self::Variables>;
}

impl<'de, T> QueryBuilder<'de> for T
where
    T: QueryFragment<'de>,
    T::SchemaType: crate::schema::QueryRoot,
{
    type Variables = T::Variables;

    fn build(vars: Self::Variables) -> Operation<T, Self::Variables> {
        Operation::<T, T::Variables>::query(vars)
    }
}

// TODO: update mutation builder to support new query structure stuff...

/// Provides a `build` function on `QueryFragment`s that represent a mutation
pub trait MutationBuilder<'de>: Sized {
    /// The type that this mutation takes as variables.
    /// May be `()` if no variables are accepted.
    type Variables;

    /// Constructs a mutation operation for this QueryFragment.
    fn build(args: Self::Variables) -> Operation<Self, Self::Variables>;
}

impl<'de, T> MutationBuilder<'de> for T
where
    T: QueryFragment<'de>,
    T::SchemaType: crate::schema::MutationRoot,
{
    type Variables = T::Variables;

    fn build(vars: Self::Variables) -> Operation<Self, Self::Variables> {
        Operation::<T, T::Variables>::mutation(vars)
    }
}

/// Provides a `build` function on `QueryFragment`s that represent a subscription
pub trait SubscriptionBuilder<'a>: Sized {
    /// The type that this subscription takes as variables.
    /// May be `()` if no variables are accepted.
    type Variables;

    /// Constructs a subscription operation for this QueryFragment.
    fn build(vars: Self::Variables) -> StreamingOperation<Self, Self::Variables>;
}

impl<'de, T> SubscriptionBuilder<'de> for T
where
    T: QueryFragment<'de>,
    T::SchemaType: crate::schema::SubscriptionRoot,
{
    type Variables = T::Variables;

    /// Constructs a subscription operation for this QueryFragment.
    fn build(vars: Self::Variables) -> StreamingOperation<Self, T::Variables> {
        StreamingOperation::<T, T::Variables>::subscription(vars)
    }
}
