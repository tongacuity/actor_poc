use async_trait::async_trait;
use std::marker::Send;
use tokio::task;

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
}
