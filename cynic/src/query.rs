use crate::{
    selection_set, FragmentArguments, GraphQLResponse, QueryBody, QueryRoot, SelectionSet,
};

pub struct Query<'a, ResponseData> {
    selection_set: SelectionSet<'a, ResponseData, ()>,
}

impl<'a, ResponseData> Query<'a, ResponseData>
where
    ResponseData: 'a,
{
    pub fn new<Root: QueryRoot>(selection_set: SelectionSet<'a, ResponseData, Root>) -> Self {
        Query {
            selection_set: selection_set::query_root(selection_set),
        }
    }

    pub fn body<'b>(&'b self) -> Result<QueryBody<'b>, ()> {
        self.selection_set
            .query_and_arguments()
            .map(|(query, arguments)| {
                let variables = arguments
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| (format!("_{}", i), value))
                    .collect();

                QueryBody { query, variables }
            })
    }

    pub fn decode_response(
        &self,
        response: GraphQLResponse<serde_json::Value>,
    ) -> Result<GraphQLResponse<ResponseData>, json_decode::DecodeError> {
        if let Some(data) = response.data {
            Ok(GraphQLResponse {
                // TODO: GET RID OF UNWRAP.  I am being extremely lazy by calling it.
                data: Some(self.selection_set.decode(&data).unwrap()),
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
