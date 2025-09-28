use std::{sync::Arc, time::Duration};

use async_graphql_axum::GraphQLSubscription;
use axum::{Router, routing::post};
use builder::MockGraphQlServerBuilder;
use handler::graphql_handler;
use url::Url;

use crate::{ReceivedRequest, Schema, SchemaExecutor};

pub(crate) mod builder;
mod handler;

pub struct MockGraphQlServer {
    state: AppState,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    port: u16,
}

impl Drop for MockGraphQlServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            shutdown.send(()).ok();
        }
    }
}

impl MockGraphQlServer {
    pub(crate) fn builder(schema: impl Schema + 'static) -> MockGraphQlServerBuilder {
        MockGraphQlServerBuilder::new(Arc::new(schema))
    }

    async fn new_impl(schema: Arc<dyn Schema>, port: Option<u16>) -> Self {
        let state = AppState {
            schema: schema.clone(),
            received_requests: Default::default(),
        };

        let app = Router::new()
            .route("/", post(graphql_handler))
            .route_service("/ws", GraphQLSubscription::new(SchemaExecutor(schema)))
            .with_state(state.clone());

        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port.unwrap_or(0)))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();

        let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel::<()>();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    shutdown_receiver.await.ok();
                })
                .await
                .unwrap();
        });

        // Give the server time to start
        tokio::time::sleep(Duration::from_millis(20)).await;

        MockGraphQlServer {
            state,
            shutdown: Some(shutdown_sender),
            port,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn url(&self) -> Url {
        format!("http://127.0.0.1:{}", self.port).parse().unwrap()
    }

    pub fn sdl(&self) -> String {
        self.state.schema.sdl()
    }

    pub fn websocket_url(&self) -> Url {
        format!("ws://127.0.0.1:{}/ws", self.port).parse().unwrap()
    }

    pub fn drain_received_requests(&self) -> impl Iterator<Item = ReceivedRequest> + '_ {
        std::iter::from_fn(|| self.state.received_requests.pop())
    }
}

#[derive(Clone)]
struct AppState {
    schema: Arc<dyn Schema>,
    received_requests: Arc<crossbeam_queue::SegQueue<ReceivedRequest>>,
}
