//! HTTP client support for cynic.
//!
//! These are hidden behind feature flags by default as HTTP clients are quite
//! heavy dependencies, and there's several options to choose from.

#[cfg(feature = "surf")]
pub use self::surf_ext::SurfExt;

#[cfg(feature = "reqwest")]
pub use reqwest_ext::ReqwestExt;

#[cfg(feature = "reqwest-blocking")]
pub use reqwest_blocking_ext::ReqwestBlockingExt;

#[cfg(feature = "surf")]
mod surf_ext {
    use serde_json::json;
    use std::{future::Future, pin::Pin};

    use crate::{GraphQLResponse, Operation};

    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    /// An extension trait for surf::RequestBuilder.
    ///
    /// ```rust,no_run
    /// # mod query_dsl {
    /// #   cynic::query_dsl!("../schemas/starwars.schema.graphql");
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #    schema_path = "../schemas/starwars.schema.graphql",
    /// #    query_module = "query_dsl",
    /// #    graphql_type = "Film"
    /// # )]
    /// # struct Film {
    /// #    title: Option<String>,
    /// #    director: Option<String>
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #     schema_path = "../schemas/starwars.schema.graphql",
    /// #     query_module = "query_dsl",
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
    /// let response = surf::post("https://swapi-graphql.netlify.com/.netlify/functions/index")
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
    pub trait SurfExt {
        /// Runs a GraphQL query with the parameters in RequestBuilder, decodes
        /// the and returns the result.
        ///
        /// If a `json_decode::Error` occurs it can be obtained via downcast_ref on
        /// the `surf::Error`.
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQLResponse<ResponseData>, surf::Error>>;
    }

    impl SurfExt for surf::RequestBuilder {
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQLResponse<ResponseData>, surf::Error>> {
            Box::pin(async move {
                self.body(json!(&operation))
                    .recv_json::<GraphQLResponse<serde_json::Value>>()
                    .await
                    .and_then(|response| operation.decode_response(response).map_err(|e| e.into()))
            })
        }
    }
}

#[cfg(any(feature = "reqwest", feature = "reqwest-blocking"))]
#[derive(thiserror::Error, Debug)]
pub enum CynicReqwestError {
    #[error("Error making HTTP request: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Error decoding GraphQL response: {0}")]
    DecodeError(#[from] json_decode::DecodeError),
}

#[cfg(feature = "reqwest")]
mod reqwest_ext {
    use super::CynicReqwestError;
    use std::{future::Future, pin::Pin};

    use crate::{GraphQLResponse, Operation};

    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    /// An extension trait for reqwest::RequestBuilder.
    ///
    /// ```rust,no_run
    /// # mod query_dsl {
    /// #   cynic::query_dsl!("../schemas/starwars.schema.graphql");
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #    schema_path = "../schemas/starwars.schema.graphql",
    /// #    query_module = "query_dsl",
    /// #    graphql_type = "Film"
    /// # )]
    /// # struct Film {
    /// #    title: Option<String>,
    /// #    director: Option<String>
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #     schema_path = "../schemas/starwars.schema.graphql",
    /// #     query_module = "query_dsl",
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
    /// let response = client.post("https://swapi-graphql.netlify.com/.netlify/functions/index")
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
    pub trait ReqwestExt {
        /// Runs a GraphQL query with the parameters in RequestBuilder, decodes
        /// the and returns the result.
        ///
        /// If a `json_decode::Error` occurs it can be obtained via downcast_ref on
        /// the `surf::Error`.
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQLResponse<ResponseData>, CynicReqwestError>>;
    }

    impl ReqwestExt for reqwest::RequestBuilder {
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQLResponse<ResponseData>, CynicReqwestError>> {
            Box::pin(async move {
                match self
                    .json(&operation)
                    .send()
                    //.recv_json::<GraphQLResponse<serde_json::Value>>()
                    .await
                {
                    Ok(response) => response
                        .json::<GraphQLResponse<serde_json::Value>>()
                        .await
                        .map_err(CynicReqwestError::ReqwestError)
                        .and_then(|gql_response| {
                            operation
                                .decode_response(gql_response)
                                .map_err(CynicReqwestError::DecodeError)
                        }),
                    Err(e) => Err(CynicReqwestError::ReqwestError(e)),
                }
            })
        }
    }
}

#[cfg(feature = "reqwest-blocking")]
mod reqwest_blocking_ext {
    use super::CynicReqwestError;

    use crate::{GraphQLResponse, Operation};

    /// An extension trait for reqwest::blocking::RequestBuilder.
    ///
    /// ```rust,no_run
    /// # mod query_dsl {
    /// #   cynic::query_dsl!("../schemas/starwars.schema.graphql");
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #    schema_path = "../schemas/starwars.schema.graphql",
    /// #    query_module = "query_dsl",
    /// #    graphql_type = "Film"
    /// # )]
    /// # struct Film {
    /// #    title: Option<String>,
    /// #    director: Option<String>
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #     schema_path = "../schemas/starwars.schema.graphql",
    /// #     query_module = "query_dsl",
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
    /// let response = client.post("https://swapi-graphql.netlify.com/.netlify/functions/index")
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
    pub trait ReqwestBlockingExt {
        /// Runs a GraphQL query with the parameters in RequestBuilder, decodes
        /// the and returns the result.
        ///
        /// If a `json_decode::Error` occurs it can be obtained via downcast_ref on
        /// the `surf::Error`.
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> Result<GraphQLResponse<ResponseData>, CynicReqwestError>;
    }

    impl ReqwestBlockingExt for reqwest::blocking::RequestBuilder {
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> Result<GraphQLResponse<ResponseData>, CynicReqwestError> {
            self.json(&operation)
                .send()
                .and_then(|response| response.json::<GraphQLResponse<serde_json::Value>>())
                .map_err(CynicReqwestError::ReqwestError)
                .and_then(|gql_response| {
                    operation
                        .decode_response(gql_response)
                        .map_err(CynicReqwestError::DecodeError)
                })
        }
    }
}
