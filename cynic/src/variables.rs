//! Types & traits for working with GraphQL Variables

use std::marker::PhantomData;

use crate::queries::InputLiteral;

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

/// Allows a query variable struct to be converted to literals for easier inlining
/// into a graphql document.
///
/// Cynic can derive this automatically or you can add it to a QueryVariables struct
/// yourself.
pub trait QueryVariableLiterals {
    /// Gets an InputLiteral for the given variable from this set of variables
    fn get(&self, variable_name: &str) -> Option<InputLiteral>;
}

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

#[cfg(test)]
mod tests {
    use cynic::QueryVariableLiterals;

    #[test]
    fn query_variable_literals_is_object_safe() {
        #[derive(QueryVariableLiterals)]
        struct Blah {
            x: String,
        }

        let _: Box<dyn QueryVariableLiterals> = Box::new(Blah { x: "hello".into() });
    }
}
