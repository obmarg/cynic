use super::{Operation, QueryFragment, QueryVariables, StreamingOperation};

// TODO maybe swap into `let operation = variables.build_query();`

/// Provides a `build` function on `QueryFragment`s that represent a query
pub trait QueryBuilder<Variables>: Sized {
    /// Constructs a query operation for this QueryFragment.
    fn build(vars: Variables) -> Operation<Self, Variables>;
}

impl<T, Variables> QueryBuilder<Variables> for T
where
    Variables: QueryVariables,
    T: QueryFragment<VariablesFields = Variables::Fields>,
    T::SchemaType: crate::schema::QueryRoot,
{
    fn build(vars: Variables) -> Operation<T, Variables> {
        Operation::<T, Variables>::query(vars)
    }
}

// TODO: update mutation builder to support new query structure stuff...

/// Provides a `build` function on `QueryFragment`s that represent a mutation
pub trait MutationBuilder<Variables>: Sized {
    /// The type that this mutation takes as variables.
    /// May be `()` if no variables are accepted.

    /// Constructs a mutation operation for this QueryFragment.
    fn build(args: Variables) -> Operation<Self, Variables>;
}

impl<T, Variables> MutationBuilder<Variables> for T
where
    Variables: QueryVariables,
    T: QueryFragment<VariablesFields = Variables::Fields>,
    T::SchemaType: crate::schema::MutationRoot,
{
    fn build(vars: Variables) -> Operation<Self, Variables> {
        Operation::<T, Variables>::mutation(vars)
    }
}

/// Provides a `build` function on `QueryFragment`s that represent a subscription
pub trait SubscriptionBuilder<Variables>: Sized {
    /// The type that this subscription takes as variables.
    /// May be `()` if no variables are accepted.

    /// Constructs a subscription operation for this QueryFragment.
    fn build(vars: Variables) -> StreamingOperation<Self, Variables>;
}

impl<T, Variables> SubscriptionBuilder<Variables> for T
where
    Variables: QueryVariables,
    T: QueryFragment<VariablesFields = Variables::Fields>,
    T::SchemaType: crate::schema::SubscriptionRoot,
{
    /// Constructs a subscription operation for this QueryFragment.
    fn build(vars: Variables) -> StreamingOperation<Self, Variables> {
        StreamingOperation::<T, Variables>::subscription(vars)
    }
}
