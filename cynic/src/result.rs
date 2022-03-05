/// The response to a GraphQl operation
#[derive(Debug, serde::Deserialize)]
pub struct GraphQlResponse<T, ErrorExtensions = serde::de::IgnoredAny> {
    /// The operation data (if the operation was succesful)
    pub data: Option<T>,

    /// Any errors that occurred as part of this operation
    pub errors: Option<Vec<GraphQlError<ErrorExtensions>>>,
}

/// A model describing an error which has taken place during execution.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, thiserror::Error)]
#[error("{message}")]
pub struct GraphQlError<Extensions> {
    /// A description of the error which has taken place.
    pub message: String,
    /// Optional description of the locations where the errors have taken place.
    pub locations: Option<Vec<GraphQlErrorLocation>>,
    /// Optional path to the response field which experienced the associated error.
    pub path: Option<Vec<GraphQlErrorPathSegment>>,
    /// Optional arbitrary extra data describing the error in more detail.
    pub extensions: Option<Extensions>,
}

impl<ErrorExtensions> GraphQlError<ErrorExtensions> {
    /// Construct a new instance.
    pub fn new(
        message: String,
        locations: Option<Vec<GraphQlErrorLocation>>,
        path: Option<Vec<GraphQlErrorPathSegment>>,
        extensions: Option<ErrorExtensions>,
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

/// A segment of a GraphQL error path.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum GraphQlErrorPathSegment {
    /// A path segment representing a field by name.
    Field(String),
    /// A path segment representing an index offset, zero-based.
    Index(i32),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_default_graphql_response_ignores_extensions() {
        let response = json!({
            "data": null,
            "errors": [{
                "message": "hello",
                "locations": null,
                "path": null,
                "extensions": {"some": "string"}
            }]
        });
        insta::assert_debug_snapshot!(serde_json::from_value::<GraphQlResponse<()>>(response).unwrap(), @r###"
        GraphQlResponse {
            data: None,
            errors: Some(
                [
                    GraphQlError {
                        message: "hello",
                        locations: None,
                        path: None,
                        extensions: Some(
                            IgnoredAny,
                        ),
                    },
                ],
            ),
        }
        "###);
    }
}
