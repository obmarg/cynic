//! HTTP client support for cynic.
//!
//! These are hidden behind feature flags by default as HTTP clients are quite
//! heavy dependencies, and there's several options to choose from.

#[cfg(feature = "http-surf")]
#[cfg_attr(docsrs, doc(cfg(feature = "surf")))]
pub use self::surf_ext::SurfExt;

#[cfg(feature = "http-reqwest")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
pub use reqwest_ext::ReqwestExt;

#[cfg(feature = "http-reqwest-blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest-blocking")))]
pub use reqwest_blocking_ext::ReqwestBlockingExt;

#[cfg(feature = "http-surf")]
mod surf_ext {
    use serde_json::json;
    use std::{future::Future, pin::Pin};

    use crate::{GraphQlResponse, Operation};

    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    /// An extension trait for surf::RequestBuilder.
    ///
    /// ```rust,no_run
    /// # mod schema {
    /// #   cynic::use_schema!("../schemas/starwars.schema.graphql");
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #    schema_path = "../schemas/starwars.schema.graphql",
    /// #    schema_module = "schema",
    /// # )]
    /// # struct Film {
    /// #    title: Option<String>,
    /// #    director: Option<String>
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #     schema_path = "../schemas/starwars.schema.graphql",
    /// #     schema_module = "schema",
    /// #     graphql_type = "Root"
    /// # )]
    /// # struct FilmDirectorQuery {
    /// #     #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
    /// #     film: Option<Film>,
    /// # }
    /// use cynic::{http::SurfExt, QueryBuilder};
    ///
    /// # async move {
    /// let operation = FilmDirectorQuery::build(());
    ///
    /// let response = surf::post("https://swapi-graphql.netlify.app/.netlify/functions/index")
    ///     .run_graphql(operation)
    ///     .await
    ///     .unwrap();
    ///
    /// println!(
    ///     "The director is {}",
    ///     response.data
    ///         .and_then(|d| d.film)
    ///         .and_then(|f| f.director)
    ///         .unwrap()
    /// );
    /// # };
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "http-surf")))]
    pub trait SurfExt {
        /// Runs a GraphQL query with the parameters in RequestBuilder, deserializes
        /// the response and returns the result.
        fn run_graphql<ResponseData, Vars>(
            self,
            operation: Operation<ResponseData, Vars>,
        ) -> BoxFuture<'static, Result<GraphQlResponse<ResponseData>, surf::Error>>
        where
            Vars: serde::Serialize,
            ResponseData: serde::de::DeserializeOwned + 'static;
    }

    impl SurfExt for surf::RequestBuilder {
        fn run_graphql<ResponseData, Vars>(
            self,
            operation: Operation<ResponseData, Vars>,
        ) -> BoxFuture<'static, Result<GraphQlResponse<ResponseData>, surf::Error>>
        where
            Vars: serde::Serialize,
            ResponseData: serde::de::DeserializeOwned + 'static,
        {
            let operation = json!(&operation);
            Box::pin(async move {
                let mut response = self.body(operation).await?;

                if !response.status().is_success() {
                    let body_string = response.body_string().await?;
                    match serde_json::from_str::<GraphQlResponse<ResponseData>>(&body_string) {
                        Ok(response) => return Ok(response),
                        Err(_) => {
                            return Err(surf::Error::from_str(
                                response.status(),
                                format!("Server returned {}: {}", response.status(), body_string),
                            ))
                        }
                    };
                }

                response.body_json::<GraphQlResponse<ResponseData>>().await
            })
        }
    }
}

/// The error type returned by `ReqwestExt` & `ReqwestBlockingExt`
#[cfg(any(feature = "http-reqwest", feature = "http-reqwest-blocking"))]
#[derive(thiserror::Error, Debug)]
pub enum CynicReqwestError {
    /// An error from reqwest when making an HTTP request.
    #[error("Error making HTTP request: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// An error response from the server with the given status code and body.
    #[error("Server returned {0}: {1}")]
    ErrorResponse(reqwest::StatusCode, String),
}

#[cfg(feature = "http-reqwest")]
mod reqwest_ext {
    use super::CynicReqwestError;
    use std::{future::Future, marker::PhantomData, pin::Pin};

    use crate::{GraphQlResponse, Operation};

    #[cfg(not(target_arch = "wasm32"))]
    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    #[cfg(target_arch = "wasm32")]
    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

    /// An extension trait for reqwest::RequestBuilder.
    ///
    /// ```rust,no_run
    /// # mod schema {
    /// #   cynic::use_schema!("../schemas/starwars.schema.graphql");
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #    schema_path = "../schemas/starwars.schema.graphql",
    /// #    schema_module = "schema",
    /// # )]
    /// # struct Film {
    /// #    title: Option<String>,
    /// #    director: Option<String>
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #     schema_path = "../schemas/starwars.schema.graphql",
    /// #     schema_module = "schema",
    /// #     graphql_type = "Root"
    /// # )]
    /// # struct FilmDirectorQuery {
    /// #     #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
    /// #     film: Option<Film>,
    /// # }
    /// use cynic::{http::ReqwestExt, QueryBuilder};
    ///
    /// # async move {
    /// let operation = FilmDirectorQuery::build(());
    ///
    /// let client = reqwest::Client::new();
    /// let response = client.post("https://swapi-graphql.netlify.app/.netlify/functions/index")
    ///     .run_graphql(operation)
    ///     .await
    ///     .unwrap();
    ///
    /// println!(
    ///     "The director is {}",
    ///     response.data
    ///         .and_then(|d| d.film)
    ///         .and_then(|f| f.director)
    ///         .unwrap()
    /// );
    /// # };
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "http-reqwest")))]
    pub trait ReqwestExt {
        /// Runs a GraphQL query with the parameters in RequestBuilder, deserializes
        /// the and returns the result.
        fn run_graphql<ResponseData, Vars>(
            self,
            operation: Operation<ResponseData, Vars>,
        ) -> CynicReqwestBuilder<ResponseData>
        where
            Vars: serde::Serialize,
            ResponseData: serde::de::DeserializeOwned + 'static;
    }

    /// A builder for cynics reqwest integration
    ///
    /// Implements `IntoFuture`, users should `.await` the builder or call
    /// `into_future` directly when they're ready to send the request.
    pub struct CynicReqwestBuilder<ResponseData, ErrorExtensions = serde::de::IgnoredAny> {
        builder: reqwest::RequestBuilder,
        _marker: std::marker::PhantomData<fn() -> (ResponseData, ErrorExtensions)>,
    }

    impl<ResponseData, Errors> CynicReqwestBuilder<ResponseData, Errors> {
        pub fn new(builder: reqwest::RequestBuilder) -> Self {
            Self {
                builder,
                _marker: std::marker::PhantomData,
            }
        }
    }

    impl<ResponseData: serde::de::DeserializeOwned, Errors: serde::de::DeserializeOwned>
        std::future::IntoFuture for CynicReqwestBuilder<ResponseData, Errors>
    {
        type Output = Result<GraphQlResponse<ResponseData, Errors>, CynicReqwestError>;

        type IntoFuture =
            BoxFuture<'static, Result<GraphQlResponse<ResponseData, Errors>, CynicReqwestError>>;

        fn into_future(self) -> Self::IntoFuture {
            Box::pin(async move {
                let http_result = self.builder.send().await;
                deser_gql(http_result).await
            })
        }
    }

    impl<ResponseData> CynicReqwestBuilder<ResponseData, serde::de::IgnoredAny> {
        /// Sets the type that will be deserialized for the extensions fields of any errors in the response
        pub fn retain_extensions<ErrorExtensions>(
            self,
        ) -> CynicReqwestBuilder<ResponseData, ErrorExtensions>
        where
            ErrorExtensions: serde::de::DeserializeOwned,
        {
            let CynicReqwestBuilder { builder, _marker } = self;

            CynicReqwestBuilder {
                builder,
                _marker: PhantomData,
            }
        }
    }

    async fn deser_gql<ResponseData, ErrorExtensions>(
        response: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<GraphQlResponse<ResponseData, ErrorExtensions>, CynicReqwestError>
    where
        ResponseData: serde::de::DeserializeOwned,
        ErrorExtensions: serde::de::DeserializeOwned,
    {
        let response = match response {
            Ok(response) => response,
            Err(e) => return Err(CynicReqwestError::ReqwestError(e)),
        };

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await;
            let text = match text {
                Ok(text) => text,
                Err(e) => return Err(CynicReqwestError::ReqwestError(e)),
            };

            let Ok(deserred) = serde_json::from_str(&text) else {
                let response = CynicReqwestError::ErrorResponse(status, text);
                return Err(response);
            };

            Ok(deserred)
        } else {
            let json = response.json().await;
            json.map_err(CynicReqwestError::ReqwestError)
        }
    }

    impl ReqwestExt for reqwest::RequestBuilder {
        fn run_graphql<ResponseData, Vars>(
            self,
            operation: Operation<ResponseData, Vars>,
        ) -> CynicReqwestBuilder<ResponseData>
        where
            Vars: serde::Serialize,
            ResponseData: serde::de::DeserializeOwned + 'static,
        {
            CynicReqwestBuilder::new(self.json(&operation))
        }
    }
}

#[cfg(feature = "http-reqwest-blocking")]
mod reqwest_blocking_ext {
    use super::CynicReqwestError;

    use crate::{GraphQlResponse, Operation};

    /// An extension trait for reqwest::blocking::RequestBuilder.
    ///
    /// ```rust,no_run
    /// # mod schema {
    /// #   cynic::use_schema!("../schemas/starwars.schema.graphql");
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #    schema_path = "../schemas/starwars.schema.graphql",
    /// #    schema_module = "schema",
    /// # )]
    /// # struct Film {
    /// #    title: Option<String>,
    /// #    director: Option<String>
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #     schema_path = "../schemas/starwars.schema.graphql",
    /// #     schema_module = "schema",
    /// #     graphql_type = "Root"
    /// # )]
    /// # struct FilmDirectorQuery {
    /// #     #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
    /// #     film: Option<Film>,
    /// # }
    /// use cynic::{http::ReqwestBlockingExt, QueryBuilder};
    ///
    /// let operation = FilmDirectorQuery::build(());
    ///
    /// let client = reqwest::blocking::Client::new();
    /// let response = client.post("https://swapi-graphql.netlify.app/.netlify/functions/index")
    ///     .run_graphql(operation)
    ///     .unwrap();
    ///
    /// println!(
    ///     "The director is {}",
    ///     response.data
    ///         .and_then(|d| d.film)
    ///         .and_then(|f| f.director)
    ///         .unwrap()
    /// );
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "http-reqwest-blocking")))]
    pub trait ReqwestBlockingExt {
        /// Runs a GraphQL query with the parameters in RequestBuilder, deserializes
        /// the and returns the result.
        fn run_graphql<ResponseData, Vars>(
            self,
            operation: Operation<ResponseData, Vars>,
        ) -> Result<GraphQlResponse<ResponseData>, CynicReqwestError>
        where
            Vars: serde::Serialize,
            ResponseData: serde::de::DeserializeOwned + 'static;
    }

    impl ReqwestBlockingExt for reqwest::blocking::RequestBuilder {
        fn run_graphql<ResponseData, Vars>(
            self,
            operation: Operation<ResponseData, Vars>,
        ) -> Result<GraphQlResponse<ResponseData>, CynicReqwestError>
        where
            Vars: serde::Serialize,
            ResponseData: serde::de::DeserializeOwned + 'static,
        {
            let response = self.json(&operation).send()?;

            let status = response.status();
            if !status.is_success() {
                let body_string = response.text().map_err(CynicReqwestError::ReqwestError)?;

                match serde_json::from_str::<GraphQlResponse<ResponseData>>(&body_string) {
                    Ok(response) => return Ok(response),
                    Err(_) => {
                        return Err(CynicReqwestError::ErrorResponse(status, body_string));
                    }
                };
            }

            Ok(response.json::<GraphQlResponse<ResponseData>>()?)
        }
    }
}
