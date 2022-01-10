use std::marker::PhantomData;

use crate::{
    core::QueryFragment,
    queries::{QueryBuilder, SelectionSet},
    schema::QueryRoot,
};

/// An Operation that can be sent to a remote GraphQL server.
///
/// This contains a GraphQL query string and variable HashMap.  It can be
/// serialized into JSON with `serde::Serialize` and sent to a remote server,
/// and has a `decode_response` function that knows how to decode a response.
#[derive(serde::Serialize)]
pub struct Operation<QueryFragment> {
    /// The graphql query string that will be sent to the server
    pub query: String,

    #[serde(skip)]
    phantom: PhantomData<fn() -> QueryFragment>,
    // The variables that will be sent to the server as part of this operation
    // pub variables: HashMap<String, Argument>,
}

impl<'de, Fragment> Operation<Fragment>
where
    Fragment: QueryFragment<'de>,
    Fragment::SchemaType: QueryRoot,
{
    /// Constructs a new Operation from a query `SelectionSet`
    pub fn query() -> Self {
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
            phantom: PhantomData,
        }
    }
}

// /// Constructs a new Operation from a mutation `SelectionSet`
// pub fn mutation<Root: MutationRoot>(
//     selection_set: SelectionSet<'a, ResponseData, Root>,
// ) -> Self {
//     let (query, arguments, decoder) =
//         mutation_root(selection_set).query_arguments_and_decoder();

//     let variables = arguments
//         .into_iter()
//         .enumerate()
//         .map(|(i, a)| (format!("_{}", i), a))
//         .collect();

//     Operation {
//         query,
//         variables,
//         decoder,
//     }
// }

// TODO: StreamingOperation etc.
