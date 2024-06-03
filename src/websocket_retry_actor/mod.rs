use async_trait::async_trait;
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_util::sync::CancellationToken;

use crate::actor::websocket_actor::{WebSocketActor, WebsocketActorHandler};

pub enum WebSocketRetryActorMessage {
    Get {
        response: oneshot::Sender<Option<String>>,
    },
}

const WS_ENDPOINT: &str = "ws://127.0.0.1:8080/";

pub struct WebSocketRetryActor {
    pub name: String,
    pub receiver: mpsc::Receiver<WebSocketRetryActorMessage>,
    cancellation_token: CancellationToken,
}

#[async_trait]
impl WebSocketActor for WebSocketRetryActor {
    async fn initialize(&mut self) {
        println!("{}: Initializing WebSocketRetryActor", self.name);
    }

    async fn connect(
        &mut self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::Error>
    {
        let url = url::Url::parse(WS_ENDPOINT).unwrap();
        let ws_stream = connect_async(url).await;
        match ws_stream {
            Ok((ws_stream, _)) => Ok(ws_stream),
            Err(e) => Err(e),
        }
    }

    async fn handle_event(
        &mut self,
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> bool {
        let (mut _write, mut read) = ws_stream.split();

        let mut local_bbo_data: Option<String> = None;

        let need_retry = loop {
            tokio::select! {
                Some(message) = read.next() => {
                    match message {
                        Ok(data) => {
                            match  data {
                                Message::Close(_) => {
                                    println!("********************************* Close. Let restart the connection");
                                    break true;
                                }
                                Message::Text(text) => {
                                    println!(".....{} received text: {}", self.name, text);
                                    local_bbo_data = Some(text.clone());
                                }
                                _ => ()
                            }
                        }
                        Err(e) => {
                            println!("********************************* Error: {:?}", e);
                            break true;
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

                _ = self.cancellation_token.cancelled() => {
                    println!("-----{}: is cancelled", self.name);
                    break false;
                },
            }
        };

        need_retry
    }
}

impl WebSocketRetryActor {
    pub fn new(
        name: String,
        receiver: mpsc::Receiver<WebSocketRetryActorMessage>,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            name,
            receiver,
            cancellation_token,
        }
    }
}

#[derive(Clone)]
pub struct WebSocketRetryActorHandler {
    sender: mpsc::Sender<WebSocketRetryActorMessage>,
    cancellation_token: CancellationToken,
}

impl WebsocketActorHandler for WebSocketRetryActorHandler {
    fn get_cancellation_token(&self) -> Option<tokio_util::sync::CancellationToken> {
        Some(self.cancellation_token.clone())
    }
}

impl WebSocketRetryActorHandler {
    pub fn new(
        sender: mpsc::Sender<WebSocketRetryActorMessage>,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            sender,
            cancellation_token,
        }
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
