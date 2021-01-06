use std::{collections::HashMap, sync::Arc};

use async_executors::{JoinHandle, SpawnHandle, SpawnHandleExt};
use async_tungstenite::{
    tungstenite::{self, Message},
    WebSocketStream,
};
use futures::{
    channel::{mpsc, oneshot},
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

// TODO: Docstrings, book etc.

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

        let (shutdown_sender, shutdown_receiver) = oneshot::channel();

        let (receiver_sink, receiver_stream) = stream.split();
        let operations = Arc::new(Mutex::new(HashMap::new()));

        let receiver_handle = runtime
            .spawn_handle(receiver_loop::<S>(
                receiver_stream,
                Arc::clone(&operations),
                shutdown_sender,
            ))
            .unwrap();

        let (sender_sink, sender_stream) = mpsc::channel(1);

        let sender_handle = runtime
            .spawn_handle(sender_loop(
                sender_stream,
                receiver_sink,
                Arc::clone(&operations),
                shutdown_receiver,
            ))
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

        // TODO: This needs to return a type that
        // has close & some sort of status func on it.
        // Have the receiver send details and have that intercepted
        // by this type and stored.
        receiver.map(move |response| op.decode_response(response).unwrap())
    }
}

type OperationSender = mpsc::Sender<GraphQLResponse<serde_json::Value>>;

type OperationMap = Arc<Mutex<HashMap<Uuid, OperationSender>>>;

// TODO: Think about whether there's actually some Arc cycles here
// that I need to care about

async fn receiver_loop<S>(
    mut receiver: futures::stream::SplitStream<WebSocketStream<S>>,
    operations: OperationMap,
    shutdown: oneshot::Sender<()>,
) where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin + Send,
{
    // TODO: Ok, so basically need a oneshot from here -> sender that
    // tells the sender to stop.  It can close it's incoming, drain it's stream
    // and then close the streams in the HashMap.
    while let Some(msg) = receiver.next().await {
        println!("Received message: {:?}", msg);
        if handle_message(msg, &operations).await.is_err() {
            println!("Error happened, killing things");
            break;
        }
    }

    shutdown.send(()).expect("Couldn't shutdown sender");
}

async fn handle_message(
    msg: Result<Message, tungstenite::Error>,
    operations: &OperationMap,
) -> Result<(), ()> {
    // TODO Make the unwraps into ?
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

    Ok(())
}

async fn sender_loop<S>(
    message_stream: mpsc::Receiver<Message>,
    mut ws_sender: futures::stream::SplitSink<WebSocketStream<S>, Message>,
    operations: OperationMap,
    shutdown: oneshot::Receiver<()>,
) where
    S: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    use futures::{future::FutureExt, select};

    let mut message_stream = message_stream.fuse();
    let mut shutdown = shutdown.fuse();

    loop {
        select! {
            msg = message_stream.next() => {
                if let Some(msg) = msg {
                    println!("Sending message: {:?}", msg);
                    ws_sender.send(msg).await.unwrap();
                } else {
                    // TODO: Do I need to indicate errors in here to the rest of the system?
                    return;
                }
            }
            _ = shutdown => {
                // Shutdown the incoming message stream
                let mut message_stream = message_stream.into_inner();
                message_stream.close();
                while message_stream.next().await.is_some() {}

                // Clear out any operations
                operations.lock().await.clear();

                return;
            }
        }
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
    // TODO: Need to allow the client to just use stream & sink directly.
    // That way I can impl tests for it indepdendant of tungsten stuff...

    // TODO: tests of shutdown behaviour etc would be good.
    // also mocked tests and what not
}
