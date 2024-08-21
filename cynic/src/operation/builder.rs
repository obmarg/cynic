use std::{borrow::Cow, collections::HashSet, marker::PhantomData};

use crate::{
    queries::{build_executable_document, OperationType},
    schema::{MutationRoot, QueryRoot, SubscriptionRoot},
    QueryFragment, QueryVariables,
};

use super::Operation;

/// Low level builder for [Operation].
///
/// Users should prefer to use [crate::QueryBuilder], [crate::MutationBuilder] or
/// [crate::SubscriptionBuilder] where possible, unless they need the control offered by
/// this builder.
pub struct OperationBuilder<QueryFragment, Variables = ()> {
    variables: Option<Variables>,
    operation_kind: OperationType,
    operation_name: Option<Cow<'static, str>>,
    features: HashSet<String>,
    phantom: PhantomData<fn() -> QueryFragment>,
}

impl<Fragment, Variables> OperationBuilder<Fragment, Variables>
where
    Fragment: QueryFragment,
    Variables: QueryVariables,
{
    fn new(operation_kind: OperationType) -> Self {
        OperationBuilder {
            variables: None,
            operation_kind,
            operation_name: Fragment::name(),
            features: HashSet::new(),
            phantom: PhantomData,
        }
    }

    /// Creates an `OperationBuilder` for a query operation
    pub fn query() -> Self
    where
        Fragment::SchemaType: QueryRoot,
    {
        Self::new(OperationType::Query)
    }

    /// Creates an `OperationBuilder` for a mutation operation
    pub fn mutation() -> Self
    where
        Fragment::SchemaType: MutationRoot,
    {
        Self::new(OperationType::Mutation)
    }

    /// Creates an `OperationBuilder` for a subscription operation
    pub fn subscription() -> Self
    where
        Fragment::SchemaType: SubscriptionRoot,
    {
        Self::new(OperationType::Subscription)
    }

    /// Adds variables for the operation
    pub fn with_variables(self, variables: Variables) -> Self {
        Self {
            variables: Some(variables),
            ..self
        }
    }

    /// Sets variables for the operation
    pub fn set_variables(&mut self, variables: Variables) {
        self.variables = Some(variables);
    }

    /// Enables a feature for the operation
    pub fn with_feature_enabled(mut self, feature: &str) -> Self {
        self.enable_feature(feature);
        self
    }

    /// Sets an enabled feature for the operation
    pub fn enable_feature(&mut self, feature: &str) {
        self.features.insert(feature.to_string());
    }

    /// Adds a name to the operation
    pub fn with_operation_name(self, name: &str) -> Self {
        OperationBuilder {
            operation_name: Some(Cow::Owned(name.to_string())),
            ..self
        }
    }

    /// Sets a name for the operation
    pub fn set_operation_name(&mut self, name: &str) {
        self.operation_name = Some(Cow::Owned(name.to_string()));
    }

    /// Tries to builds an [Operation]
    pub fn build(self) -> Result<super::Operation<Fragment, Variables>, OperationBuildError> {
        Ok(Operation {
            query: build_executable_document::<Fragment, Variables>(
                self.operation_kind,
                self.operation_name.as_deref(),
                self.features.clone(),
            ),
            variables: self.variables.ok_or(OperationBuildError::VariablesNotSet)?,
            operation_name: self.operation_name,
            phantom: PhantomData,
        })
    }
}

#[derive(thiserror::Error, Debug)]
/// Errors that can occur when building the operation
pub enum OperationBuildError {
    #[error("You need to call with_variables or set_variables before calling build")]
    /// Error for when `set_variables` or `with_variables` was not called
    VariablesNotSet,
    #[error("Couldn't format the query into a string: {0}")]
    /// Error when a write! call that builds the query string failed
    CouldntBuildQueryString(#[from] std::fmt::Error),
}
