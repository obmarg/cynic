use std::collections::HashSet;

use crate::{
    queries::{build_executable_document, OperationType},
    OperationBuilder,
};

use super::{Operation, QueryFragment, QueryVariables, StreamingOperation};

/// Provides a `build` function on `QueryFragment`s that represent a query
pub trait QueryBuilder<Variables>: Sized {
    /// Constructs a query operation for this QueryFragment.
    fn build(vars: Variables) -> Operation<Self, Variables>;

    /// Creates an operation builder for this query
    fn operation_builder(vars: Variables) -> OperationBuilder<Self, Variables>;

    /// Returns the executable document for this query
    fn executable_document() -> String;
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

    fn operation_builder(vars: Variables) -> OperationBuilder<Self, Variables> {
        OperationBuilder::query().with_variables(vars)
    }

    fn executable_document() -> String {
        build_executable_document::<T, Variables>(
            OperationType::Query,
            T::name().as_deref(),
            HashSet::new(),
        )
    }
}

/// Provides a `build` function on `QueryFragment`s that represent a mutation
pub trait MutationBuilder<Variables>: Sized {
    /// The type that this mutation takes as variables.
    /// May be `()` if no variables are accepted.

    /// Constructs a mutation operation for this QueryFragment.
    fn build(args: Variables) -> Operation<Self, Variables>;

    /// Creates an operation buidler for this mutation
    fn operation_builder(vars: Variables) -> OperationBuilder<Self, Variables>;

    /// Returns the executable document for this mutation
    fn executable_document() -> String;
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

    fn operation_builder(vars: Variables) -> OperationBuilder<Self, Variables> {
        OperationBuilder::mutation().with_variables(vars)
    }

    fn executable_document() -> String {
        build_executable_document::<T, Variables>(
            OperationType::Mutation,
            T::name().as_deref(),
            HashSet::new(),
        )
    }
}

/// Provides a `build` function on `QueryFragment`s that represent a subscription
pub trait SubscriptionBuilder<Variables>: Sized {
    /// The type that this subscription takes as variables.
    /// May be `()` if no variables are accepted.

    /// Constructs a subscription operation for this QueryFragment.
    fn build(vars: Variables) -> StreamingOperation<Self, Variables>;

    /// Creates an operation buidler for this subscription
    fn operation_builder(vars: Variables) -> OperationBuilder<Self, Variables>;

    /// Returns the executable document for this subscription
    fn executable_document() -> String;
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

    fn operation_builder(vars: Variables) -> OperationBuilder<Self, Variables> {
        OperationBuilder::subscription().with_variables(vars)
    }

    fn executable_document() -> String {
        build_executable_document::<T, Variables>(
            OperationType::Subscription,
            T::name().as_deref(),
            HashSet::new(),
        )
    }
}
