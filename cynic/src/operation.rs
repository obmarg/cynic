use json_decode::BoxDecoder;
use std::collections::HashMap;

use crate::{
    selection_set::{mutation_root, query_root, subscription_root},
    Argument, GraphQLResponse, MutationRoot, QueryRoot, SelectionSet, SubscriptionRoot,
};

/// An Operation that can be sent to a remote GraphQL server.
///
/// This contains a GraphQL query string and variable HashMap.  It can be
/// serialized into JSON with `serde::Serialize` and sent to a remote server,
/// and has a `decode_response` function that knows how to decode a response.
#[derive(serde::Serialize)]
pub struct Operation<'a, ResponseData> {
    pub query: String,
    pub variables: HashMap<String, Argument>,
    #[serde(skip)]
    decoder: BoxDecoder<'a, ResponseData>,
}

impl<'a, ResponseData: 'a> Operation<'a, ResponseData> {
    /// Constructs a new Operation from a query `SelectionSet`
    pub fn query<Root: QueryRoot>(selection_set: SelectionSet<'a, ResponseData, Root>) -> Self {
        let (query, arguments, decoder) = query_root(selection_set).query_arguments_and_decoder();

        let variables = arguments
            .into_iter()
            .enumerate()
            .map(|(i, a)| (format!("_{}", i), a))
            .collect();

        Operation {
            query,
            variables,
            decoder,
        }
    }

    /// Constructs a new Operation from a mutation `SelectionSet`
    pub fn mutation<Root: MutationRoot>(
        selection_set: SelectionSet<'a, ResponseData, Root>,
    ) -> Self {
        let (query, arguments, decoder) =
            mutation_root(selection_set).query_arguments_and_decoder();

        let variables = arguments
            .into_iter()
            .enumerate()
            .map(|(i, a)| (format!("_{}", i), a))
            .collect();

        Operation {
            query,
            variables,
            decoder,
        }
    }

    /// Decodes a response.  Note that you need to decode a GraphQLResponse
    /// from JSON before passing to this function
    pub fn decode_response(
        &self,
        response: GraphQLResponse<serde_json::Value>,
    ) -> Result<GraphQLResponse<ResponseData>, json_decode::DecodeError> {
        if let Some(data) = response.data {
            Ok(GraphQLResponse {
                data: Some(self.decoder.decode(&data)?),
                errors: response.errors,
            })
        } else {
            Ok(GraphQLResponse {
                data: None,
                errors: response.errors,
            })
        }
    }
}

/// A StreamingOperation is an Operation that expects a stream of results.
///
/// Currently this is means subscriptions.
pub struct StreamingOperation<'a, ResponseData> {
    pub(crate) inner: Operation<'a, ResponseData>,
}

impl<'a, ResponseData: 'a> StreamingOperation<'a, ResponseData> {
    /// Constructs a new Operation from a query `SelectionSet`
    pub fn subscription<Root: SubscriptionRoot>(
        selection_set: SelectionSet<'a, ResponseData, Root>,
    ) -> Self {
        let (query, arguments, decoder) =
            subscription_root(selection_set).query_arguments_and_decoder();

        let variables = arguments
            .into_iter()
            .enumerate()
            .map(|(i, a)| (format!("_{}", i), a))
            .collect();

        StreamingOperation {
            inner: Operation {
                query,
                variables,
                decoder,
            },
        }
    }

    pub fn decode_response(
        &self,
        response: GraphQLResponse<serde_json::Value>,
    ) -> Result<GraphQLResponse<ResponseData>, json_decode::DecodeError> {
        self.inner.decode_response(response)
    }
}

impl<ResponseData> serde::Serialize for StreamingOperation<'_, ResponseData> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}
