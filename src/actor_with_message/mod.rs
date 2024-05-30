use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::actor::{Actor, ActorHandler};

#[derive(Debug)]
pub enum ActorMessage {
    Get { value: u32 },
}

pub struct ActorWithMessage {
    pub name: String,
    receiver: mpsc::Receiver<ActorMessage>,
}

#[async_trait]
impl Actor for ActorWithMessage {
    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg);
        }
    }
}

impl ActorWithMessage {
    pub fn new(name: String, receiver: mpsc::Receiver<ActorMessage>) -> Self {
        Self { name, receiver }
    }

    fn handle_message(&self, msg: ActorMessage) {
        println!("+++++{} received message {:?}", self.name, msg);
    }
}

#[derive(Clone)]
pub struct ActorWithMessageHandler {
    sender: mpsc::Sender<ActorMessage>,
}

impl ActorHandler for ActorWithMessageHandler {}

impl ActorWithMessageHandler {
    pub fn new(sender: mpsc::Sender<ActorMessage>) -> Self {
        Self { sender }
    }

    pub async fn send_message(&self, msg: ActorMessage) {
        self.sender.send(msg).await.unwrap();
    }
}
