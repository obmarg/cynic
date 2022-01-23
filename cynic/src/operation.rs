use std::marker::PhantomData;

use crate::{
    core::QueryFragment,
    queries::{QueryBuilder, SelectionSet},
    schema::{MutationRoot, QueryRoot},
};

/// An Operation that can be sent to a remote GraphQL server.
///
/// This contains a GraphQL query string and variable HashMap.  It can be
/// serialized into JSON with `serde::Serialize` and sent to a remote server.
pub struct Operation<QueryFragment, Variables = ()> {
    /// The graphql query string that will be sent to the server
    pub query: String,

    pub variables: Variables,

    phantom: PhantomData<fn() -> QueryFragment>,
    // The variables that will be sent to the server as part of this operation
    // pub variables: HashMap<String, Argument>,
}

impl<QueryFragment, Variables> serde::Serialize for Operation<QueryFragment, Variables> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: impl this.  Think about how best to serialize variables
        todo!()
    }
}

impl<'de, Fragment, Variables> Operation<Fragment, Variables>
where
    Fragment: QueryFragment<'de>,
{
    /// Constructs a new Operation for a query
    pub fn query(variables: Variables) -> Self
    where
        Fragment::SchemaType: QueryRoot,
    {
        use std::fmt::Write;

        let mut selection_set = SelectionSet::default();
        let builder = QueryBuilder::new(&mut selection_set);
        Fragment::query(builder);

        // TODO: Somehow enforce Variable type

        // TODO: There has to be a better way to do this/place to structure this.
        // At the least a QueryRoot: std::fmt::Display type.
        let mut query = String::new();
        writeln!(&mut query, "query{}", selection_set);

        // TODO: Handle arguments and what not.

        Operation {
            query,
            variables,
            phantom: PhantomData,
        }
    }

    /// Constructs a new Operation for a mutation
    pub fn mutation(variables: Variables) -> Self
    where
        Fragment::SchemaType: MutationRoot,
    {
        use std::fmt::Write;

        let mut selection_set = SelectionSet::default();
        let builder = QueryBuilder::new(&mut selection_set);
        Fragment::query(builder);

        // TODO: There has to be a better way to do this/place to structure this.
        // At the least a QueryRoot: std::fmt::Display type.
        let mut query = String::new();
        writeln!(&mut query, "query{}", selection_set);

        // TODO: Handle arguments and what not.

        Operation {
            query,
            variables,
            phantom: PhantomData,
        }
    }
}

// TODO: StreamingOperation etc.
