/// The response to a GraphQl operation
#[derive(Debug, Clone)]
pub struct GraphQlResponse<T> {
    /// The operation data (if the operation was successful)
    pub data: Option<T>,

    /// Any errors that occurred as part of this operation
    pub errors: Option<Vec<GraphQlError>>,

    /// Optional arbitrary extra data describing the error in more detail.
    extensions: Option<serde_json::Value>,
}

impl<T> GraphQlResponse<T> {
    /// Deserialize the extensions field on this response as an instance of E
    pub fn extensions<'a, E>(&'a self) -> Result<E, serde_json::Error>
    where
        E: serde::Deserialize<'a>,
    {
        let Some(extensions) = self.extensions.as_ref() else {
            return E::deserialize(serde_json::Value::Null);
        };

        E::deserialize(extensions)
    }
}

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
    /// Optional arbitrary extra data describing the error in more detail.
    extensions: Option<serde_json::Value>,
}

impl GraphQlError {
    /// Construct a new instance.
    pub fn new(
        message: String,
        locations: Option<Vec<GraphQlErrorLocation>>,
        path: Option<Vec<GraphQlErrorPathSegment>>,
    ) -> Self {
        GraphQlError {
            message,
            locations,
            path,
            extensions: None,
        }
    }

    /// Populate the extensions field of this error
    pub fn with_extensions<E>(mut self, extensions: E) -> Result<Self, serde_json::Error>
    where
        E: serde::Serialize,
    {
        self.set_extensions(extensions)?;
        Ok(self)
    }

    /// Populate the extensions field of this error
    pub fn set_extensions<E>(&mut self, extensions: E) -> Result<(), serde_json::Error>
    where
        E: serde::Serialize,
    {
        self.extensions = Some(serde_json::to_value(extensions)?);
        Ok(())
    }

    /// Deserialize the extensions field on this error as an instance of E
    pub fn extensions<'a, E>(&'a self) -> Result<E, serde_json::Error>
    where
        E: serde::Deserialize<'a>,
    {
        let Some(extensions) = self.extensions.as_ref() else {
            return E::deserialize(serde_json::Value::Null);
        };

        E::deserialize(extensions)
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

impl<'de, T> serde::Deserialize<'de> for GraphQlResponse<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(serde::Deserialize)]
        struct ResponseDeser<T> {
            /// The operation data (if the operation was successful)
            data: Option<T>,

            /// Any errors that occurred as part of this operation
            errors: Option<Vec<GraphQlError>>,

            extensions: Option<serde_json::Value>,
        }

        let ResponseDeser {
            data,
            errors,
            extensions,
        } = ResponseDeser::deserialize(deserializer)?;

        if data.is_none() && errors.is_none() {
            return Err(D::Error::custom(
                "Either data or errors must be present in a GraphQL response",
            ));
        }

        Ok(GraphQlResponse {
            data,
            errors,
            extensions,
        })
    }
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

    #[test]
    fn test_graphql_response_fails_on_completely_invalid_response() {
        let response = json!({
            "message": "This endpoint requires you to be authenticated.",
        });
        serde_json::from_value::<GraphQlResponse<()>>(response).unwrap_err();
    }
}
