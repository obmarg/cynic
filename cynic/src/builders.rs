use super::{Operation, QueryFragment};

pub trait QueryBuilder<'de>: Sized {
    /// The type that this query takes as arguments.
    /// May be `()` if no arguments are accepted.
    type Variables;

    /// Constructs a query operation for this QueryFragment.
    fn build(vars: Self::Variables) -> Operation<Self, Self::Variables>;
}

impl<'de, T> QueryBuilder<'de> for T
where
    T: crate::core::QueryFragment<'de>,
    T::SchemaType: crate::schema::QueryRoot,
{
    type Variables = T::Variables;

    fn build(vars: Self::Variables) -> Operation<T, T::Variables> {
        Operation::<T, T::Variables>::query(vars)
    }
}

// TODO: update mutation builder to support new query structure stuff...

/// Provides a `build` function on `QueryFragment`s that represent a mutation
pub trait MutationBuilder<'de>: Sized {
    /// The type that this mutation takes as arguments.
    /// May be `()` if no arguments are accepted.
    type Variables;

    /// Constructs a mutation operation for this QueryFragment.
    fn build(args: Self::Variables) -> Operation<Self, Self::Variables>;
}

impl<'de, T> MutationBuilder<'de> for T
where
    T: crate::core::QueryFragment<'de>,
    T::SchemaType: crate::schema::MutationRoot,
{
    type Variables = T::Variables;

    fn build(vars: Self::Variables) -> Operation<Self, T::Variables> {
        // TODO: handle args
        Operation::<T, T::Variables>::mutation(vars)
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
