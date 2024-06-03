use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::actor::{Actor, ActorHandler};

pub enum WebSocketRetryActorMessage {
    Get {
        response: oneshot::Sender<Option<String>>,
    },
}

const WS_ENDPOINT: &str = "ws://127.0.0.1:8080/";

pub struct WebSocketRetryActor {
    pub name: String,
    pub receiver: mpsc::Receiver<WebSocketRetryActorMessage>,
}

#[async_trait]
impl Actor for WebSocketRetryActor {
    async fn run(&mut self) {
        let local_bbo_data: Option<String> = Option::None;

        loop {
            let (mut _write, mut read) = self.connect().await.split();

            loop {
                tokio::select! {
                    Some(message) = read.next() => {
                        println!("Received message: {:?}", message);
                        match message {
                            Ok(data) => {
                                match  data {
                                    Message::Close(_) => {
                                        println!("********************************* Close. Let restart the connection");
                                        break;
                                    }
                                    Message::Text(text) => {
                                        println!(".....{} received text {}", self.name, text);
                                    }
                                    _ => ()
                                }
                            }
                            Err(e) => {
                                println!("********************************* Error: {:?}", e);
                            }
                        }
                    },

                    Some(message) = self.receiver.recv() => {
                        match message {
                            WebSocketRetryActorMessage::Get{response} => {
                                _ = response.send(local_bbo_data.clone());
                            }
                        }
                    },

                    else => break,
                }
            }
        }
    }
}

impl WebSocketRetryActor {
    pub fn new(name: String, receiver: mpsc::Receiver<WebSocketRetryActorMessage>) -> Self {
        Self { name, receiver }
    }

    pub async fn connect(&mut self) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>{
        let url = url::Url::parse(WS_ENDPOINT).unwrap();
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        println!(
            "{}: WebSocket handshake has been successfully completed",
            self.name
        );

        ws_stream
    }
}

#[derive(Clone)]
pub struct WebSocketRetryActorHandler {
    sender: mpsc::Sender<WebSocketRetryActorMessage>,
}

impl ActorHandler for WebSocketRetryActorHandler {
    fn get_cancellation_token(&self) -> Option<tokio_util::sync::CancellationToken> {
        None
    }
}

impl WebSocketRetryActorHandler {
    pub fn new(sender: mpsc::Sender<WebSocketRetryActorMessage>) -> Self {
        Self { sender }
    }

    pub async fn get_data(&self) -> Option<String> {
        let (sender, receiver) = oneshot::channel();
        match self
            .sender
            .send(WebSocketRetryActorMessage::Get { response: (sender) })
            .await
        {
            Ok(_) => {
                if let Ok(data) = receiver.await {
                    data
                } else {
                    None
                }
            }

            Err(_) => None,
        }
    }
}
