//! Tools for building GraphQL queries in cynic

mod ast;
mod builders;
mod flatten;
mod indent;
mod input_literal_ser;
mod recurse;
mod type_eq;
mod variables;

use variables::VariableDefinitions;

use crate::QueryVariableLiterals;

pub use self::{
    ast::{Argument, InputLiteral, SelectionSet},
    builders::{SelectionBuilder, VariableMatch},
    flatten::FlattensInto,
    input_literal_ser::to_input_literal,
    recurse::Recursable,
    type_eq::IsFieldType,
};

use std::{collections::HashSet, rc::Rc, sync::mpsc};

/// Builds an executable document for the given Fragment
///
/// Users should prefer to use `crate::QueryBuilder`, `crate::MutationBuilder` &
/// `crate::SubscriptionBuilder` over this function, but may prefer to use this
/// function when they need to construct an executable document without first constructing an
/// `Operation` (e.g. when they do not have easy access to a variable struct)
///
/// Note that this function does not enforce as much safety as the regular
/// builders and relies on the user providing the right `r#type` and correct variables
/// when submitting the query.
pub fn build_executable_document<Fragment, Variables>(
    r#type: OperationType,
    operation_name: Option<&str>,
    features_enabled: HashSet<String>,
    inline_variables: Option<&dyn QueryVariableLiterals>,
) -> String
where
    Fragment: crate::QueryFragment,
    Variables: crate::QueryVariables,
{
    let features_enabled = Rc::new(features_enabled);
    let mut selection_set = SelectionSet::default();
    let (variable_tx, variable_rx) = mpsc::channel();
    let builder = SelectionBuilder::<_, Fragment::VariablesFields>::new(
        &mut selection_set,
        &variable_tx,
        &features_enabled,
        inline_variables,
    );

    Fragment::query(builder);

    let vars = VariableDefinitions::new::<Variables>(variable_rx.try_iter().collect());

    let name_str = operation_name.unwrap_or("");

    let declaration_str = r#type.as_str();

    format!("{declaration_str} {name_str}{vars}{selection_set}")
}

/// The kind of operation to build an executable document for
pub enum OperationType {
    /// A query operation
    Query,
    /// A mutation operation
    Mutation,
    /// A subscription operation
    Subscription,
}

impl OperationType {
    /// The operation type as it would appear in a GraphQl document
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationType::Query => "query",
            OperationType::Mutation => "mutation",
            OperationType::Subscription => "subscription",
        }
    }
}
