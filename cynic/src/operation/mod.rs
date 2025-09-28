use std::{borrow::Cow, marker::PhantomData};

use crate::{
    core::QueryFragment,
    schema::{MutationRoot, QueryRoot, SubscriptionRoot},
    QueryVariables,
};

mod builder;

pub use builder::{OperationBuildError, OperationBuilder};

/// An Operation that can be sent to a remote GraphQL server.
///
/// This contains a GraphQL query string and variable HashMap.  It can be
/// serialized into JSON with `serde::Serialize` and sent to a remote server.
pub struct Operation<QueryFragment, Variables = ()> {
    /// The graphql query string that will be sent to the server
    pub query: String,

    /// The variables that will be sent to the server as part of this operation
    pub variables: Variables,

    /// The name of the operation in query that we should run
    pub operation_name: Option<Cow<'static, str>>,

    phantom: PhantomData<fn() -> QueryFragment>,
}

impl<F, V> Clone for Operation<F, V>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            query: self.query.clone(),
            variables: self.variables.clone(),
            operation_name: self.operation_name.clone(),
            phantom: PhantomData,
        }
    }
}

impl<QueryFragment, Variables> std::fmt::Debug for Operation<QueryFragment, Variables>
where
    Variables: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Operation")
            .field("query", &self.query)
            .field("variables", &self.variables)
            .field("operation_name", &self.operation_name)
            .finish()
    }
}

impl<QueryFragment, Variables> serde::Serialize for Operation<QueryFragment, Variables>
where
    Variables: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map_serializer = serializer.serialize_map(Some(2))?;
        map_serializer.serialize_entry("query", &self.query)?;
        map_serializer.serialize_entry("variables", &self.variables)?;
        if let Some(operation_name) = &self.operation_name {
            map_serializer.serialize_entry("operationName", &operation_name)?;
        }
        map_serializer.end()
    }
}

impl<Fragment, Variables> Operation<Fragment, Variables>
where
    Fragment: QueryFragment,
    Variables: QueryVariables,
{
    /// Constructs a new operation from a String & some variables.
    ///
    /// This is useful for certain testing cirumstances, but offers no typesafety.
    /// [crate::QueryBuilder], [crate::MutationBuilder] and [crate::SubscriptionBuilder]
    /// should be preferered.
    pub fn new(query: String, variables: Variables) -> Self {
        Operation {
            query,
            variables,
            operation_name: None,
            phantom: PhantomData,
        }
    }

    /// Constructs a new Operation for a query
    pub fn query(variables: Variables) -> Self
    where
        Fragment::SchemaType: QueryRoot,
    {
        OperationBuilder::query()
            .with_variables(variables)
            .build()
            .expect("to be able to build query")
    }

    /// Constructs a new Operation for a mutation
    pub fn mutation(variables: Variables) -> Self
    where
        Fragment::SchemaType: MutationRoot,
    {
        OperationBuilder::mutation()
            .with_variables(variables)
            .build()
            .expect("to be able to build mutation")
    }
}

impl<F, V> AsRef<Operation<F, V>> for Operation<F, V> {
    fn as_ref(&self) -> &Operation<F, V> {
        self
    }
}

/// A StreamingOperation is an Operation that expects a stream of results.
///
/// Currently this is means subscriptions.
pub struct StreamingOperation<ResponseData, Variables = ()> {
    inner: Operation<ResponseData, Variables>,
}

impl<Fragment, Variables> StreamingOperation<Fragment, Variables>
where
    Fragment: QueryFragment,
    Variables: QueryVariables,
{
    /// Constructs a new Operation for a subscription
    pub fn subscription(variables: Variables) -> Self
    where
        Fragment::SchemaType: SubscriptionRoot,
    {
        let inner = OperationBuilder::subscription()
            .with_variables(variables)
            .build()
            .expect("to be able to build subscription");

        StreamingOperation { inner }
    }
}

impl<ResponseData, Variables> serde::Serialize for StreamingOperation<ResponseData, Variables>
where
    Variables: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}
