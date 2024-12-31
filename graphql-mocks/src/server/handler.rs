use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, response::IntoResponse};
use http::HeaderMap;

use crate::ReceivedRequest;

use super::AppState;

pub(super) async fn graphql_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> axum::response::Response {
    let req = req.into_inner();

    // Record the request incase tests want to inspect it.
    // async_graphql::Request isn't clone so we do a deser roundtrip instead
    state.received_requests.push(ReceivedRequest {
        headers: headers.clone(),
        body: serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap(),
    });

    let headers = headers
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                String::from_utf8_lossy(value.as_bytes()).to_string(),
            )
        })
        .collect();

    let response: GraphQLResponse = state.schema.execute(headers, req).await.into();

    response.into_response()
}
