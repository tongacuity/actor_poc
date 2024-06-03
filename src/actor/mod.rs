pub mod websocket_actor;

use async_trait::async_trait;
use std::marker::Send;
use tokio::task;
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait Actor {
    async fn run(&mut self);
}

pub trait ActorHandler {
    fn start<T: Actor + Send + 'static>(&self, actor: T) {
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
