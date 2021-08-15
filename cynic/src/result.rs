/// The result of a GraphQL Operation.
///
/// Either it fully succeeds, or we have some errors.  If we have some errors
/// then some fields might unexpectedly be null, so deserialization _might_ have
/// failed.  We represent that with a PossiblyParsedData<T>.
pub type GraphQlResult<T> = Result<T, (PossiblyParsedData<T>, Vec<GraphQlError>)>;

#[deprecated(
    since = "0.13.0",
    note = "GraphQLResult has been deprecated in favour of GraphQlResult"
)]
#[allow(clippy::upper_case_acronyms)]
pub type GraphQLResult<T> = GraphQlResult<T>;

#[derive(Debug, serde::Deserialize)]
pub struct GraphQlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, serde::Deserialize, thiserror::Error)]
pub enum CynicError<E: std::fmt::Debug + std::error::Error> {
    #[error("Query Error")]
    Query(Vec<GraphQlError>),
    #[error("Request Error")]
    Request(E),
}

#[deprecated(
    since = "0.13.0",
    note = "GraphQLResponse has been deprecated in favour of GraphQlResponse"
)]
#[allow(clippy::upper_case_acronyms)]
pub type GraphQLResponse<T> = GraphQlResponse<T>;

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

/// A model describing an error which has taken place during execution.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, thiserror::Error)]
#[error("{message}")]
pub struct GraphQlError {
    /// A description of the error which has taken place.
    pub message: String,
    /// Optional description of the locations where the errors have taken place.
    pub locations: Option<Vec<GraphQlErrorLocation>>,
    /// Optional path to the response field which experienced the associated error.
    pub path: Option<Vec<GraphQlErrorPathSegment>>,
    /// Optional arbitrary JSON data describing the error in more detail.
    pub extensions: Option<serde_json::Value>,
}

#[deprecated(
    since = "0.13.0",
    note = "GraphQlError has been deprecated in favour of GraphQlError"
)]
#[allow(clippy::upper_case_acronyms)]
pub type GraphQLError = GraphQlError;

impl GraphQlError {
    /// Construct a new instance.
    pub fn new(
        message: String,
        locations: Option<Vec<GraphQlErrorLocation>>,
        path: Option<Vec<GraphQlErrorPathSegment>>,
        extensions: Option<serde_json::Value>,
    ) -> Self {
        GraphQlError {
            message,
            locations,
            path,
            extensions,
        }
    }
}

/// A line and column offset describing the location of an error within a GraphQL document.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct GraphQlErrorLocation {
    /// The line at which the associated error begins.
    pub line: i32,
    /// The column of the line at which the associated error begins.
    pub column: i32,
}

#[deprecated(
    since = "0.13.0",
    note = "GraphQlErrorLocation has been deprecated in favour of GraphQlErrorLocation"
)]
#[allow(clippy::upper_case_acronyms, dead_code)]
pub type GraphQLErrorLocation = GraphQlErrorLocation;

/// A segment of a GraphQL error path.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum GraphQlErrorPathSegment {
    /// A path segment representing a field by name.
    Field(String),
    /// A path segment representing an index offset, zero-based.
    Index(i32),
}

#[deprecated(
    since = "0.13.0",
    note = "GraphQlErrorPathSegment has been deprecated in favour of GraphQlErrorPathSegment"
)]
#[allow(clippy::upper_case_acronyms, dead_code)]
pub type GraphQLErrorPathSegment = GraphQlErrorPathSegment;

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
