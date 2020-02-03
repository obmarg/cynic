use std::collections::HashMap;

mod argument;
mod field;
mod query;
mod result;
mod scalar;
pub mod selection_set;

pub use argument::Argument;
pub use query::Query;
pub use result::{GraphQLError, GraphQLResponse, GraphQLResult, PossiblyParsedData};
pub use scalar::Scalar;
pub use selection_set::SelectionSet;

pub trait QueryFragment {
    type SelectionSet;
    type Arguments: FragmentArguments;

    fn fragment(arguments: Self::Arguments) -> Self::SelectionSet;
}

/// A marker trait for the arguments types on QueryFragments.
///
/// We use this in combination with the IntoArguments trait below
/// to convert between different argument types in a query heirarchy.
pub trait FragmentArguments {}

impl FragmentArguments for () {}

/// Used for converting between different argument types in a QueryFragment
/// heirarchy.
///
/// For example if an outer QueryFragment has a struct with several parameters
/// but an inner QueryFragment needs none then we can use () as the arguments
/// type on the inner fragments and use the blanket implementation of IntoArguments
/// to convert to ().
///
/// Similarly, the
pub trait IntoArguments<T> {
    fn into_args(&self) -> T;
}

impl IntoArguments<()> for dyn FragmentArguments {
    fn into_args(&self) -> () {
        ()
    }
}

impl<T> IntoArguments<T> for T
where
    T: Clone,
{
    fn into_args(&self) -> T {
        // TODO: Figure out if there's a way to avoid this clone...
        self.clone()
    }
}

pub trait QueryRoot {}

#[derive(Debug, serde::Serialize)]
pub struct QueryBody<'a> {
    query: String,
    variables: HashMap<String, &'a serde_json::Value>,
}

pub use cynic_codegen::{query_dsl, FragmentArguments, QueryFragment};
