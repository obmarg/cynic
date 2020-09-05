use json_decode::BoxDecoder;
use std::collections::HashMap;

use crate::{
    selection_set::{mutation_root, query_root},
    Argument, GraphQLResponse, MutationRoot, QueryRoot, SelectionSet,
};

#[derive(serde::Serialize)]
pub struct Operation<'a, ResponseData> {
    pub query: String,
    pub variables: HashMap<String, Argument>,
    #[serde(skip)]
    decoder: BoxDecoder<'a, ResponseData>,
}

impl<'a, ResponseData: 'a> Operation<'a, ResponseData> {
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
