mod builder;
mod resolvers;

pub use self::builder::DynamicSchemaBuilder;

use futures_lite::stream;

/// async-graphql powered dynamic schemas for tests.
///
/// Occasionally its just easier to write SDL & resolvers, this lets you do that.
pub struct DynamicSchema {
    schema: async_graphql::dynamic::Schema,
    sdl: String,
}

impl DynamicSchema {
    pub fn builder(sdl: impl AsRef<str>) -> DynamicSchemaBuilder {
        DynamicSchemaBuilder::new(sdl.as_ref())
    }
}

#[async_trait::async_trait]
impl super::Schema for DynamicSchema {
    async fn execute(
        &self,
        _headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        self.schema.execute(request).await
    }

    fn execute_stream(
        &self,
        request: async_graphql::Request,
    ) -> stream::Boxed<async_graphql::Response> {
        Box::pin(self.schema.execute_stream(request))
    }

    fn sdl(&self) -> String {
        self.sdl.clone()
    }
}
