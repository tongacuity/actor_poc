use async_trait::async_trait;
use std::marker::Send;
use tokio::net::TcpStream;
use tokio::task;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait WebSocketActor {
    async fn initialize(&mut self) {}
    async fn run(&mut self) {
        self.initialize().await;

        loop {
            if let Ok(ws_stream) = self.connect().await {
                let need_reconnect = self.handle_event(ws_stream).await;
                if !need_reconnect {
                    break;
                }
            } else {
                panic!("Failed to connect to websocket server. Exiting...");
            }
        }
    }
    async fn connect(
        &mut self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::Error>;
    async fn handle_event(&mut self, ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>)
        -> bool;
}

pub trait WebsocketActorHandler {
    fn start<T: WebSocketActor + Send + 'static>(&self, actor: T) {
        let mut actor = actor;
        let _handle = task::spawn(async move {
            actor.run().await;
        });
    }

    fn get_cancellation_token(&self) -> Option<CancellationToken>;

    fn stop(&self) {
        if let Some(token) = self.get_cancellation_token() {
            token.cancel();
        }
    }
}
