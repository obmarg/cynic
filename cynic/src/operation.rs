use std::marker::PhantomData;

use crate::{
    core::QueryFragment,
    queries::{QueryBuilder, SelectionSet},
    schema::{MutationRoot, QueryRoot},
    variables::{VariableDefinition, VariableType},
    QueryVariables,
};

/// An Operation that can be sent to a remote GraphQL server.
///
/// This contains a GraphQL query string and variable HashMap.  It can be
/// serialized into JSON with `serde::Serialize` and sent to a remote server.
pub struct Operation<QueryFragment, Variables = ()> {
    /// The graphql query string that will be sent to the server
    pub query: String,

    // The variables that will be sent to the server as part of this operation
    pub variables: Variables,

    phantom: PhantomData<fn() -> QueryFragment>,
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
        map_serializer.end()
    }
}

impl<'de, Fragment, Variables> Operation<Fragment, Variables>
where
    Fragment: QueryFragment<'de>,
    Variables: QueryVariables,
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

        let vars = VariableDefinitions::new::<Variables>();

        // TODO: Somehow enforce Variable type

        // TODO: There has to be a better way to do this/place to structure this.
        // At the least a QueryRoot: std::fmt::Display type.
        let mut query = String::new();
        writeln!(&mut query, "query{vars}{selection_set}");

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

        let vars = VariableDefinitions::new::<Variables>();

        // TODO: There has to be a better way to do this/place to structure this.
        // At the least a QueryRoot: std::fmt::Display type.
        let mut query = String::new();
        writeln!(&mut query, "query{vars}{selection_set}");

        // TODO: Handle arguments and what not.

        Operation {
            query,
            variables,
            phantom: PhantomData,
        }
    }
}

// TODO: StreamingOperation etc.

struct VariableDefinitions {
    vars: &'static [(&'static str, VariableType)],
}

impl VariableDefinitions {
    fn new<T: QueryVariables>() -> Self {
        VariableDefinitions { vars: T::VARIABLES }
    }
}

impl std::fmt::Display for VariableDefinitions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.vars.is_empty() {
            return Ok(());
        }

        write!(f, "(")?;
        for (name, ty) in self.vars {
            let ty = GraphqlVariableType::new(*ty);
            write!(f, "${name}: {ty}")?;
        }
        write!(f, ")")
    }
}

enum GraphqlVariableType {
    List(Box<GraphqlVariableType>),
    NotNull(Box<GraphqlVariableType>),
    Named(&'static str),
}

impl GraphqlVariableType {
    fn new(ty: VariableType) -> Self {
        fn recurse(ty: VariableType, required: bool) -> GraphqlVariableType {
            match (ty, required) {
                (VariableType::Nullable(inner), _) => recurse(*inner, false),
                (any, true) => GraphqlVariableType::NotNull(Box::new(recurse(any, false))),
                (VariableType::List(inner), _) => {
                    GraphqlVariableType::List(Box::new(recurse(*inner, true)))
                }
                (VariableType::Named(name), false) => GraphqlVariableType::Named(name),
            }
        }

        recurse(ty, true)
    }
}

impl std::fmt::Display for GraphqlVariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphqlVariableType::List(inner) => write!(f, "[{inner}]"),
            GraphqlVariableType::NotNull(inner) => write!(f, "{inner}!"),
            GraphqlVariableType::Named(name) => write!(f, "{name}"),
        }
    }
}

// TODO: test the argument conversion/printing stuff...
// Also test serialization once I've got that written.
