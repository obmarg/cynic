//! A mock GraphQL server for use in tests

use futures_lite::stream;

use std::sync::Arc;

use serde::ser::SerializeMap;

mod dynamic;
pub mod mocks;
mod server;

pub use async_graphql::dynamic::ResolverContext;
pub use dynamic::{DynamicSchema, DynamicSchemaBuilder};
pub use server::{builder::MockGraphQlServerBuilder, MockGraphQlServer};

#[derive(Debug)]
pub struct ReceivedRequest {
    pub headers: http::HeaderMap,
    pub body: async_graphql::Request,
}

impl serde::Serialize for ReceivedRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        let mut headers = self
            .headers
            .iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    String::from_utf8_lossy(value.as_bytes()).into_owned(),
                )
            })
            .collect::<Vec<_>>();
        headers.sort_unstable();
        map.serialize_entry("headers", &headers)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl std::ops::Deref for ReceivedRequest {
    type Target = async_graphql::Request;
    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

/// Creating a trait for schema so we can use it as a trait object and avoid
/// making everything generic over Query, Mutation & Subscription params
#[async_trait::async_trait]
pub trait Schema: Send + Sync {
    async fn execute(
        &self,
        headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response;

    fn execute_stream(
        &self,
        request: async_graphql::Request,
    ) -> stream::Boxed<async_graphql::Response>;

    fn sdl(&self) -> String;
}

#[derive(Clone)]
pub struct SchemaExecutor(Arc<dyn Schema>);

#[async_trait::async_trait]
impl async_graphql::Executor for SchemaExecutor {
    /// Execute a GraphQL query.
    async fn execute(&self, request: async_graphql::Request) -> async_graphql::Response {
        self.0.execute(Default::default(), request).await
    }

    /// Execute a GraphQL subscription with session data.
    fn execute_stream(
        &self,
        request: async_graphql::Request,
        _session_data: Option<Arc<async_graphql::Data>>,
    ) -> stream::Boxed<async_graphql::Response> {
        self.0.execute_stream(request)
    }
}
