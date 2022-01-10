use super::{
    FragmentContext, MutationRoot, Operation, Operation2, QueryFragment, QueryRoot, SelectionSet,
    StreamingOperation, SubscriptionRoot,
};

use std::borrow::Borrow;

/// Provides a `build` function on `QueryFragment`s that represent a query
pub trait QueryBuilder<'a> {
    /// The type that this query takes as arguments.
    /// May be `()` if no arguments are accepted.
    type Arguments;

    /// The type that this query returns if succesful.
    type ResponseData;

    /// Constructs a query operation for this QueryFragment.
    fn build(args: impl Borrow<Self::Arguments>) -> Operation<'a, Self::ResponseData>;
}

impl<'a, T, R, Q> QueryBuilder<'a> for T
where
    T: QueryFragment<SelectionSet = SelectionSet<'a, R, Q>>,
    Q: QueryRoot,
    R: 'a,
{
    type Arguments = T::Arguments;

    type ResponseData = R;

    fn build(args: impl Borrow<Self::Arguments>) -> Operation<'a, Self::ResponseData> {
        Operation::query(Self::fragment(FragmentContext::new(args.borrow())))
    }
}

pub trait QueryBuilder2: Sized {
    /// The type that this query takes as arguments.
    /// May be `()` if no arguments are accepted.
    type Arguments;

    /// The type that this query returns if succesful.
    /// TODo: Figure out if we even need this...
    // type ResponseData;

    /// Constructs a query operation for this QueryFragment.
    fn build(args: impl Borrow<Self::Arguments>) -> Operation2<Self>;
}

impl<'de, T> QueryBuilder2 for T
where
    T: crate::core::QueryFragment<'de>,
    T::SchemaType: crate::schema::QueryRoot,
{
    // TOdO: Arguments
    type Arguments = ();

    fn build(args: impl Borrow<Self::Arguments>) -> Operation2<T> {
        Operation2::<T>::query()
    }
}

/// Provides a `build` function on `QueryFragment`s that represent a mutation
pub trait MutationBuilder<'a> {
    /// The type that this mutation takes as arguments.
    /// May be `()` if no arguments are accepted.
    type Arguments;

    /// The type that this mutation returns if succesful.
    type ResponseData;

    /// Constructs a mutation operation for this QueryFragment.
    fn build(args: impl Borrow<Self::Arguments>) -> Operation<'a, Self::ResponseData>;
}

impl<'a, T, R, Q> MutationBuilder<'a> for T
where
    T: QueryFragment<SelectionSet = SelectionSet<'a, R, Q>>,
    Q: MutationRoot,
    R: 'a,
{
    type Arguments = T::Arguments;

    type ResponseData = R;

    fn build(args: impl Borrow<Self::Arguments>) -> Operation<'a, Self::ResponseData> {
        Operation::mutation(Self::fragment(FragmentContext::new(args.borrow())))
    }
}

/// Provides a `build` function on `QueryFragment`s that represent a subscription
pub trait SubscriptionBuilder<'a> {
    /// The type that this subscription takes as arguments.
    /// May be `()` if no arguments are accepted.
    type Arguments;

    /// The type of event that will be output in the response stream of this subscription.
    type ResponseData;

    /// Constructs a subscription operation for this QueryFragment.
    fn build(args: impl Borrow<Self::Arguments>) -> StreamingOperation<'a, Self::ResponseData>;
}

impl<'a, T, R, Q> SubscriptionBuilder<'a> for T
where
    T: QueryFragment<SelectionSet = SelectionSet<'a, R, Q>>,
    Q: SubscriptionRoot,
    R: 'a,
{
    type Arguments = T::Arguments;

    type ResponseData = R;

    fn build(args: impl Borrow<Self::Arguments>) -> StreamingOperation<'a, Self::ResponseData> {
        StreamingOperation::subscription(Self::fragment(FragmentContext::new(args.borrow())))
    }
}
