//! A non functional mock of the spacex graphql api.
//!
//! I've had trouble hitting the actual API in tests, so this is useful for
//! running introspection tests.

use crate::{DynamicSchema, MockGraphQlServer};

pub async fn serve() -> MockGraphQlServer {
    DynamicSchema::builder(include_str!("../../../schemas/spacex.graphql"))
        .into_server_builder()
        .await
}
