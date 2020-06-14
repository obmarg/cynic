use json_decode::BoxDecoder;
use std::collections::HashMap;

use crate::{
    selection_set::query_root, Argument, GraphQLResponse, QueryBody, QueryRoot, SelectionSet,
};

#[derive(serde::Serialize)]
pub struct Query<'a, ResponseData> {
    pub query: String,
    pub variables: HashMap<String, Argument>,
    #[serde(skip)]
    decoder: BoxDecoder<'a, ResponseData>,
    // selection_set: SelectionSet<'a, ResponseData, ()>,
}

impl<'a, ResponseData: 'a> Query<'a, ResponseData> {
    pub fn new<Root: QueryRoot>(selection_set: SelectionSet<'a, ResponseData, Root>) -> Self {
        let (query, arguments, decoder) = query_root(selection_set).query_arguments_and_decoder();

        let variables = arguments
            .into_iter()
            .enumerate()
            .map(|(i, a)| (format!("$_{}", i), a))
            .collect();

        Query {
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
                // TODO: GET RID OF UNWRAP.  I am being extremely lazy by calling it.
                data: Some(self.decoder.decode(&data).unwrap()),
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
