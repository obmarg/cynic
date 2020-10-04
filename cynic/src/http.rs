//! HTTP client support for cynic.
//!
//! These are hidden behind feature flags by default as HTTP clients are quite
//! heavy dependencies, and there's several options to choose from.

#[cfg(feature = "surf")]
mod surf_ext {
    use serde_json::json;
    use std::{future::Future, pin::Pin};

    use crate::{GraphQLResponse, Operation};

    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

    /// An extension trait for surf::RequestBuilder.
    ///
    /// ```rust,no_run
    /// # mod query_dsl {
    /// #   cynic::query_dsl!("../examples/examples/starwars.schema.graphql");
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #    schema_path = "../examples/examples/starwars.schema.graphql",
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
    /// #     schema_path = "../examples/examples/starwars.schema.graphql",
    /// #     query_module = "query_dsl",
    /// #     graphql_type = "Root"
    /// # )]
    /// # struct FilmDirectorQuery {
    /// #     #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
    /// #     film: Option<Film>,
    /// # }
    /// use cynic::{http::SurfExt, QueryFragment};
    ///
    /// # async move {
    /// let operation = cynic::Operation::query(
    ///     FilmDirectorQuery::fragment(&())
    /// );
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
        fn run_graphql<'a, ResponseData>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQLResponse<ResponseData>, surf::Error>>
        where
            ResponseData: 'static;
    }

    impl SurfExt for surf::RequestBuilder {
        fn run_graphql<'a, ResponseData>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQLResponse<ResponseData>, surf::Error>>
        where
            ResponseData: 'static,
        {
            Box::pin(async move {
                self.body(json!(&operation))
                    .recv_json::<GraphQLResponse<serde_json::Value>>()
                    .await
                    .and_then(|response| {
                        operation
                            .decode_response(response)
                            .map_err(|e| surf::Error::new(surf::StatusCode::Ok, e))
                    })
            })
        }
    }
}

#[cfg(feature = "surf")]
pub use self::surf_ext::SurfExt;
