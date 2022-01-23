use super::{
    FragmentContext, MutationRoot, Operation, QueryFragment, QueryRoot, SelectionSet,
    SubscriptionRoot,
};

use std::borrow::Borrow;

pub trait QueryBuilder: Sized {
    /// The type that this query takes as arguments.
    /// May be `()` if no arguments are accepted.
    type Arguments;

    /// Constructs a query operation for this QueryFragment.
    fn build(args: impl Borrow<Self::Arguments>) -> Operation<Self>;
}

impl<'de, T> QueryBuilder for T
where
    T: crate::core::QueryFragment<'de>,
    T::SchemaType: crate::schema::QueryRoot,
{
    // TOdO: Arguments
    type Arguments = ();

    fn build(args: impl Borrow<Self::Arguments>) -> Operation<T> {
        Operation::<T>::query()
    }
}

// TODO: update mutation builder to support new query structure stuff...

/// Provides a `build` function on `QueryFragment`s that represent a mutation
pub trait MutationBuilder: Sized {
    /// The type that this mutation takes as arguments.
    /// May be `()` if no arguments are accepted.
    type Arguments;

    /// Constructs a mutation operation for this QueryFragment.
    fn build(args: impl Borrow<Self::Arguments>) -> Operation<Self>;
}

impl<'de, T> MutationBuilder for T
where
    T: crate::core::QueryFragment<'de>,
    T::SchemaType: crate::schema::MutationRoot,
{
    // TODO: arguments
    type Arguments = ();

    fn build(args: impl Borrow<Self::Arguments>) -> Operation<Self> {
        // TODO: handle args
        Operation::<T>::mutation()
    }
}

#[cfg(todo)]
mod todo {
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
}
