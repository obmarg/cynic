use std::marker::PhantomData;

use crate::{
    core::QueryFragment,
    queries::{SelectionBuilder, SelectionSet},
    schema::{MutationRoot, QueryRoot, SubscriptionRoot},
    variables::VariableType,
    QueryVariables,
};

/// An Operation that can be sent to a remote GraphQL server.
///
/// This contains a GraphQL query string and variable HashMap.  It can be
/// serialized into JSON with `serde::Serialize` and sent to a remote server.
pub struct Operation<QueryFragment, Variables = ()> {
    /// The graphql query string that will be sent to the server
    pub query: String,

    /// The variables that will be sent to the server as part of this operation
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
    Fragment: QueryFragment,
    Variables: QueryVariables,
{
    /// Constructs a new Operation for a query
    pub fn query(variables: Variables) -> Self
    where
        Fragment::SchemaType: QueryRoot,
    {
        use std::fmt::Write;

        let mut selection_set = SelectionSet::default();
        let builder = SelectionBuilder::<_, Fragment::VariablesFields>::new(&mut selection_set);
        Fragment::query(builder);

        let vars = VariableDefinitions::new::<Variables>();

        let mut query = String::new();
        writeln!(&mut query, "query{vars}{selection_set}").expect("Couldn't stringify query");

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
        let builder = SelectionBuilder::<_, Fragment::VariablesFields>::new(&mut selection_set);
        Fragment::query(builder);

        let vars = VariableDefinitions::new::<Variables>();

        let mut query = String::new();
        writeln!(&mut query, "mutation{vars}{selection_set}").expect("Couldn't stringify query");

        Operation {
            query,
            variables,
            phantom: PhantomData,
        }
    }
}

/// A StreamingOperation is an Operation that expects a stream of results.
///
/// Currently this is means subscriptions.
pub struct StreamingOperation<ResponseData, Variables = ()> {
    inner: Operation<ResponseData, Variables>,
}

impl<'de, Fragment, Variables> StreamingOperation<Fragment, Variables>
where
    Fragment: QueryFragment,
    Variables: QueryVariables,
{
    /// Constructs a new Operation for a subscription
    pub fn subscription(variables: Variables) -> Self
    where
        Fragment::SchemaType: SubscriptionRoot,
    {
        use std::fmt::Write;

        let mut selection_set = SelectionSet::default();
        let builder = SelectionBuilder::<_, Fragment::VariablesFields>::new(&mut selection_set);
        Fragment::query(builder);

        let vars = VariableDefinitions::new::<Variables>();

        let mut query = String::new();
        writeln!(&mut query, "subscription{vars}{selection_set}")
            .expect("Couldn't stringify query");

        StreamingOperation {
            inner: Operation {
                query,
                variables,
                phantom: PhantomData,
            },
        }
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
        let mut first = true;
        for (name, ty) in self.vars {
            if !first {
                write!(f, ", ")?;
            }
            first = false;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_printing() {
        insta::assert_display_snapshot!(VariableDefinitions {
            vars: &[
                ("foo", VariableType::List(&VariableType::Named("Foo"))),
                ("bar", VariableType::Named("Bar")),
                ("nullable_bar", VariableType::Nullable(&VariableType::Named("Bar"))),
                (
                    "nullable_list_foo",
                    VariableType::Nullable(&(VariableType::List(&VariableType::Named("Foo"))))
                ),
                (
                    "nullable_list_nullable_foo",
                    VariableType::Nullable(&VariableType::List(&VariableType::Nullable(
                        &VariableType::Named("Foo")
                    )))
                )
            ]
        }, @"($foo: [Foo!]!, $bar: Bar!, $nullable_bar: Bar, $nullable_list_foo: [Foo!], $nullable_list_nullable_foo: [Foo])")
    }
}
