use json_decode::BoxDecoder;
use serde_json::json;
use std::collections::HashMap;

use crate::{
    arguments::ArgumentWireFormat,
    selection_set::{mutation_root, query_root, subscription_root},
    Argument, GraphQlResponse, MutationRoot, QueryRoot, SelectionSet, SubscriptionRoot, Upload,
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
    pub files: Vec<(String, Upload)>,
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
            files: vec![],
            decoder,
        }
    }

    /// Constructs a new Operation from a mutation `SelectionSet`
    pub fn mutation<Root: MutationRoot>(
        selection_set: SelectionSet<'a, ResponseData, Root>,
    ) -> Self {
        let (query, arguments, decoder) =
            mutation_root(selection_set).query_arguments_and_decoder();

        let mut variables = HashMap::new();
        let mut files = Vec::new();

        for (i, argument) in arguments.into_iter().enumerate() {
            match argument.wire_format {
                ArgumentWireFormat::Serialize(_) => {
                    variables.insert(format!("_{}", i), argument);
                }
                ArgumentWireFormat::Upload(upload) => {
                    let variable_name = format!("_{}", i);
                    files.push((variable_name.clone(), upload));
                    variables.insert(
                        variable_name,
                        Argument::new(
                            &argument.name,
                            "Upload",
                            ArgumentWireFormat::Serialize(Ok(json! { Option::<()>::None })),
                        ),
                    );
                }
            }
        }

        log::debug!("vars: {:#?}", variables);

        Operation {
            query,
            variables,
            files,
            decoder,
        }
    }

    /// Decodes a response.  Note that you need to decode a GraphQlResponse
    /// from JSON before passing to this function
    pub fn decode_response(
        &self,
        response: GraphQlResponse<serde_json::Value>,
    ) -> Result<GraphQlResponse<ResponseData>, json_decode::DecodeError> {
        if let Some(data) = response.data {
            Ok(GraphQlResponse {
                data: Some(self.decoder.decode(&data)?),
                errors: response.errors,
            })
        } else {
            Ok(GraphQlResponse {
                data: None,
                errors: response.errors,
            })
        }
    }

    pub fn file_map(&self) -> HashMap<String, Vec<String>> {
        self.files
            .iter()
            .enumerate()
            // TODO: Fix
            .map(|(i, (name, _))| (i.to_string(), vec![format!("variables.{}", name)]))
            .collect()
    }
}

/// A StreamingOperation is an Operation that expects a stream of results.
///
/// Currently this is means subscriptions.
pub struct StreamingOperation<'a, ResponseData> {
    inner: Operation<'a, ResponseData>,
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
                files: vec![],
                decoder,
            },
        }
    }

    pub fn decode_response(
        &self,
        response: GraphQlResponse<serde_json::Value>,
    ) -> Result<GraphQlResponse<ResponseData>, json_decode::DecodeError> {
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
