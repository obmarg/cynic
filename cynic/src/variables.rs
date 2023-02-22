//! Types & traits for working with GraphQL Variables

use std::marker::PhantomData;

/// The type of a variable
#[derive(Debug, Clone, Copy)]
pub enum VariableType {
    /// A list of some VariableType
    List(&'static VariableType),
    /// Some VariableType that may be null
    Nullable(&'static VariableType),
    /// A type with the given name
    Named(&'static str),
}

/// Allows a struct to be used as variables in a GraphQL query.
///
/// Users should not implement this themselves, and should use the
/// `QueryVariables` derive.  All the fields on a `QueryVariables` struct should
/// be available as variables in the query, using `$variable_name` notation.
pub trait QueryVariables {
    /// A struct that determines which variables are available when using this
    /// struct.
    type Fields: QueryVariablesFields;
    /// An associated constant that contains the variable names & their types.
    ///
    /// This is used to construct the query string we send to a server.
    const VARIABLES: &'static [(&'static str, VariableType)];
}

/// Represents a set of named fields that are required for a query
pub trait QueryVariablesFields {}

impl QueryVariables for () {
    type Fields = ();
    const VARIABLES: &'static [(&'static str, VariableType)] = &[];
}
impl QueryVariablesFields for () {}

#[doc(hidden)]
/// A VariableDefinition.
///
/// These are returned by functions on the `Fields` associated type of a
/// `QueryVariables` struct.  But users shouldn't need to care about that.
pub struct VariableDefinition<Variables, Type> {
    pub name: &'static str,
    phantom: PhantomData<fn() -> (Variables, Type)>,
}

impl<Variables, Type> VariableDefinition<Variables, Type> {
    /// Create a new variable with the given name.
    pub fn new(name: &'static str) -> Self {
        VariableDefinition {
            name,
            phantom: PhantomData,
        }
    }
}
