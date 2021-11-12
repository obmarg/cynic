#[derive(Debug, serde::Deserialize)]
pub struct GraphQlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQlError>>,
}

#[deprecated(
    since = "0.13.0",
    note = "GraphQLResponse has been deprecated in favour of GraphQlResponse"
)]
#[allow(clippy::upper_case_acronyms)]
#[doc(hidden)]
pub type GraphQLResponse<T> = GraphQlResponse<T>;

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
