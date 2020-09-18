/// The result of a GraphQL Operation.
///
/// Either it fully succeeds, or we have some errors.  If we have some errors
/// then some fields might unexpectedly be null, so deserialization _might_ have
/// failed.  We represent that with a PossiblyParsedData<T>.
pub type GraphQLResult<T> = Result<T, (PossiblyParsedData<T>, Vec<GraphQLError>)>;

#[derive(Debug, serde::Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

/*
impl<T> GraphQLResponse<T> {
    fn into_result(self) -> GraphQLResult<T> {
        // TODO: I need access to the query/selection set in order to decode this.
        if let Some(errors) = self.errors {
            if !errors.is_empty() {
                // TODO: Return an error.
            }
        }

        match self {
            GraphQLResponse::Ok(t) => Ok(t),
            GraphQLResponse::Err(poss_data, errs) => Err((poss_data, errs)),
        }
    }
}*/

#[derive(Debug, serde::Deserialize, thiserror::Error)]
#[error("{message}")]
pub struct GraphQLError {
    message: String,
}

/// The data returned by a GraphQL query when the query had errors.   
/// GraphQL allows servers to return partial data in this case, but if there's
/// missing fields that aren't represented by an Option we won't have been
/// able to decode that data.
pub enum PossiblyParsedData<T> {
    ParsedData(T),
    // TODO: Could pass serde_json::Value from here, can't be bothered right
    // now though....
    UnparsedData,
}
