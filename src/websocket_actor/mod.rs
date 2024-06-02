use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::actor::{Actor, ActorHandler};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ExchangeOutgoingMessage {
    Subscribe {
        id: i32,
        method: String,
        params: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ExchangeIncomingMessage {
    Update(Update),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Update {
    #[serde(rename = "s")]
    pub symbol: Option<String>,

    #[serde(rename = "b")]
    best_bid_price: Option<Decimal>,

    #[serde(rename = "a")]
    best_ask_price: Option<Decimal>,
}

impl From<Update> for Bbo {
    fn from(value: Update) -> Self {
        Self {
            symbol: value.symbol,
            best_bid_price: value.best_bid_price,
            best_ask_price: value.best_ask_price,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Bbo {
    pub symbol: Option<String>,
    pub best_bid_price: Option<Decimal>,
    pub best_ask_price: Option<Decimal>,
}

pub enum WebSocketActorMessage {
    Get {
        response: oneshot::Sender<Option<Bbo>>,
    },
}

const WS_ENDPOINT: &str = "wss://stream.binance.com:9443/ws";

pub struct WebSocketActor {
    pub name: String,
    pub receiver: mpsc::Receiver<WebSocketActorMessage>,
}

#[async_trait]
impl Actor for WebSocketActor {
    async fn run(&mut self) {
        let mut local_bbo_data: Option<Bbo> = Option::None;

        let url = url::Url::parse(WS_ENDPOINT).unwrap();
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        println!(
            "{}: WebSocket handshake has been successfully completed",
            self.name
        );

        let (mut write, mut read) = ws_stream.split();
        let channel = String::from("btcusdt") + &String::from("@bookTicker");

        let message = Message::Text(format!(
            "{}",
            serde_json::to_string(&ExchangeOutgoingMessage::Subscribe {
                id: 1,
                method: String::from("SUBSCRIBE"),
                params: vec![channel]
            })
            .unwrap()
        ));

        write.send(message).await.unwrap();

        loop {
            tokio::select! {
                Some(message) = read.next() => {
                    if let Ok(data) = message.unwrap().into_text() {
                        if let Ok(message) = serde_json::from_str::<ExchangeIncomingMessage>(&data) {
                            match message {
                                ExchangeIncomingMessage::Update(update) => {
                                    local_bbo_data = Some(update.into());
                                }
                            };
                        }
                    }
                },

                Some(message) = self.receiver.recv() => {
                    match message {
                        WebSocketActorMessage::Get{response} => {
                            _ = response.send(local_bbo_data.clone());
                        }
                    }
                },

                else => break,
            }
        }
    }
}

impl WebSocketActor {
    pub fn new(name: String, receiver: mpsc::Receiver<WebSocketActorMessage>) -> Self {
        Self { name, receiver }
    }
}

#[derive(Clone)]
pub struct WebSocketActorHandler {
    sender: mpsc::Sender<WebSocketActorMessage>,
}

impl ActorHandler for WebSocketActorHandler {
    fn get_cancellation_token(&self) -> Option<tokio_util::sync::CancellationToken> {
        None
    }
}

impl WebSocketActorHandler {
    pub fn new(sender: mpsc::Sender<WebSocketActorMessage>) -> Self {
        Self { sender }
    }

    pub async fn get_bbo_data(&self) -> Option<Bbo> {
        let (sender, receiver) = oneshot::channel();
        match self
            .sender
            .send(WebSocketActorMessage::Get { response: (sender) })
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
