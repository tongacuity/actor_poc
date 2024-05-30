use async_trait::async_trait;
use tokio::time::{sleep, Duration};

use crate::actor::{Actor, ActorHandler};

pub struct SimpleActor {
    pub state: u32,
    pub name: String,
}

impl SimpleActor {
    pub fn new(name: String) -> SimpleActor {
        SimpleActor { state: 0, name }
    }
}

pub struct SimpleActorHandler {}

#[async_trait]
impl Actor for SimpleActor {
    async fn run(&mut self) {
        loop {
            self.state += 1;
            println!("{}: with state = {}", self.name, self.state);

            sleep(Duration::from_secs(1)).await;
        }
    }
}

impl ActorHandler for SimpleActorHandler {}

impl SimpleActorHandler {
    pub fn new() -> SimpleActorHandler {
        SimpleActorHandler {}
    }
}
