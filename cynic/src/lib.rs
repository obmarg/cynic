use std::collections::HashMap;

mod argument;
mod field;
mod result;
mod scalar;
pub mod selection_set;

pub use argument::Argument;
pub use result::{GraphQLError, GraphQLResponse, GraphQLResult, PossiblyParsedData};
pub use scalar::Scalar;
pub use selection_set::{Query, SelectionSet};

fn main() {
    println!("Hello, world!");
}

// Ok, so I need a way of figuring out TypeLocks.
// I _could_ do QueryFragment<T>
// and then do impl<T> QueryFragment<T> for X
// Though I'd need a way to constrain T.
//
// Perhaps with some kind of marker trait?
// Contains<T>?

pub trait QueryFragment {
    type SelectionSet;
    type Arguments: FragmentArguments;

    fn query(arguments: Self::Arguments) -> Self::SelectionSet;
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
    // TODO: not sure we need this next one
    //operation_name: String,
}

pub use cynic_codegen::{query_dsl, FragmentArguments, QueryFragment};
