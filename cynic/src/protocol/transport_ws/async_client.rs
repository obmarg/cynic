use std::{collections::HashMap, sync::Arc};

use async_executors::{JoinHandle, SpawnHandle, SpawnHandleExt};
use async_tungstenite::{tungstenite::Message, WebSocketStream};
use futures::{
    channel::mpsc,
    lock::Mutex,
    sink::SinkExt,
    stream::{Stream, StreamExt},
};
use uuid::Uuid;

use super::{ConnectionAck, ConnectionInit, Event};
use crate::{GraphQLResponse, Operation, StreamingOperation};

// TODO: probably want to re-export async-tungstenite from here
// and if I do that, maybe this should be a separate lib so I can release it
// independantly of cynic core?

// TODO: Make this customisable somehow.
const SUBSCRIPTION_BUFFER_SIZE: usize = 5;

pub struct AsyncWebsocketClient {
    inner: Arc<ClientInner>,
    sender_sink: mpsc::Sender<Message>,
}

impl AsyncWebsocketClient {
    // TODO: possibly make this generic over any old stream & sync type?
    // With some sort of Message abstraction to allow for websocket streams & tungstenite
    pub async fn new<S>(
        mut stream: WebSocketStream<S>,
        runtime: impl SpawnHandle<()>,
    ) -> Result<Self, ()>
    where
        S: futures::AsyncRead + futures::AsyncWrite + Unpin + 'static + Send,
    {
        // TODO: actual error handling, ditch unwraps
        stream
            .send(json_message(ConnectionInit::new()))
            .await
            .unwrap();

        match stream.next().await {
            None => todo!(),
            Some(Err(_)) => todo!(),
            Some(Ok(data)) => {
                decode_message::<ConnectionAck<()>>(data).unwrap();
                println!("Connection acked");
            }
        }

        let (receiver_sink, receiver_stream) = stream.split();
        let operations = Arc::new(Mutex::new(HashMap::new()));

        let receiver_handle = runtime
            .spawn_handle(receiver_loop::<S>(receiver_stream, Arc::clone(&operations)))
            .unwrap();

        let (sender_sink, sender_stream) = mpsc::channel(1);

        let sender_handle = runtime
            .spawn_handle(sender_loop(sender_stream, receiver_sink))
            .unwrap();

        Ok(AsyncWebsocketClient {
            inner: Arc::new(ClientInner {
                receiver_handle,
                operations,
                sender_handle,
            }),
            sender_sink,
        })
    }

    pub async fn operation<'a, T: 'a>(&self, _op: Operation<'a, T>) -> Result<(), ()> {
        todo!()
        // Probably hook into streaming operation and do a take 1 -> into_future
    }

    pub async fn streaming_operation<'a, T: 'a>(
        &mut self,
        op: StreamingOperation<'a, T>,
    ) -> impl Stream<Item = GraphQLResponse<T>> + 'a {
        let id = Uuid::new_v4();
        let (sender, receiver) = mpsc::channel(SUBSCRIPTION_BUFFER_SIZE);
        self.inner.operations.lock().await.insert(id, sender);

        let msg = json_message(super::Message::Subscribe {
            id: id.to_string(),
            payload: &op.inner,
        });

        self.sender_sink.send(msg).await.unwrap();

        // TODO: Make this closable (probably a future PR)
        receiver.map(move |response| op.decode_response(response).unwrap())
    }
}

type OperationSender = mpsc::Sender<GraphQLResponse<serde_json::Value>>;

type OperationMap = Arc<Mutex<HashMap<Uuid, OperationSender>>>;

async fn receiver_loop<S>(
    mut receiver: futures::stream::SplitStream<WebSocketStream<S>>,
    operations: OperationMap,
) where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin + Send,
{
    // TODO: how do I indicate errors in here to the rest of the client?
    // preferably in a way that allows for retries...
    while let Some(msg) = receiver.next().await {
        println!("Received message: {:?}", msg);
        let event = decode_message::<Event>(msg.unwrap()).unwrap();
        let id = &Uuid::parse_str(event.id()).unwrap();
        match event {
            Event::Next { payload, .. } => {
                let mut sink = operations.lock().await.get(&id).unwrap().clone();

                sink.send(payload).await.unwrap()
            }
            Event::Complete { .. } => {
                println!("Stream complete");
                operations.lock().await.remove(&id);
            }
            Event::Error { payload, .. } => {
                let mut sink = operations.lock().await.remove(&id).unwrap();

                sink.send(GraphQLResponse {
                    data: None,
                    errors: Some(payload),
                })
                .await
                .unwrap();
            }
        }
    }
}

async fn sender_loop<S>(
    mut message_stream: mpsc::Receiver<Message>,
    mut ws_sender: futures::stream::SplitSink<WebSocketStream<S>, Message>,
) where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    // TODO: how do I indicate errors in here to the rest of the client?
    while let Some(msg) = message_stream.next().await {
        println!("Sending message: {:?}", msg);
        ws_sender.send(msg).await.unwrap();
    }
}

struct ClientInner {
    #[allow(dead_code)]
    receiver_handle: JoinHandle<()>,
    #[allow(dead_code)]
    sender_handle: JoinHandle<()>,
    operations: OperationMap,
}

fn json_message(payload: impl serde::Serialize) -> Message {
    // TODO: dtich unwraps..
    Message::text(serde_json::to_string(&payload).unwrap())
}

fn decode_message<T: serde::de::DeserializeOwned>(message: Message) -> Result<T, ()> {
    if let Message::Text(s) = message {
        println!("Received {}", s);
        // TODO: no unwraps
        Ok(serde_json::from_str::<T>(&s).unwrap())
    } else {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // TODO: tests of shutdown behaviour etc would be good.
    // also mocked tests and what not
}
