use super::{FragmentContext, MutationRoot, Operation, QueryFragment, QueryRoot, SelectionSet};

use std::borrow::Borrow;

/// Provides a `build` function on `QueryFragment`s that represent a query
pub trait QueryBuilder<'a> {
    type Arguments;
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

/// Provides a `build` function on `QueryFragment`s that represent a mutation
pub trait MutationBuilder<'a> {
    type Arguments;
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
