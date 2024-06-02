use async_trait::async_trait;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;

use crate::actor::{Actor, ActorHandler};

pub struct SimpleActor {
    pub state: u32,
    pub name: String,
    cancellation_token: CancellationToken,
}

impl SimpleActor {
    pub fn new(name: String, cancellation_token: CancellationToken) -> SimpleActor {
        SimpleActor {
            state: 0,
            name,
            cancellation_token,
        }
    }
}

pub struct SimpleActorHandler {
    cancellation_token: CancellationToken,
}

#[async_trait]
impl Actor for SimpleActor {
    async fn run(&mut self) {
        let mut interval = interval(Duration::from_secs(1));

        loop {
            self.state += 1;

            tokio::select! {
                _ = self.cancellation_token.cancelled() => {
                    println!("-----{}: is cancelled", self.name);
                    break;
                },

                _ = interval.tick() => {
                    println!("-----{}: with state = {}", self.name, self.state);
                }
            }
        }
    }
}

impl ActorHandler for SimpleActorHandler {
    fn get_cancellation_token(&self) -> Option<CancellationToken> {
        Some(self.cancellation_token.clone())
    }
}

impl SimpleActorHandler {
    pub fn new(cancellation_token: CancellationToken) -> SimpleActorHandler {
        SimpleActorHandler { cancellation_token }
    }
}
