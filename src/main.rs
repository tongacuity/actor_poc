pub mod actor;
pub mod actor_with_message;
pub mod simple_actor;

use actor::ActorHandler;
use actor_with_message::{ActorMessage, ActorWithMessage, ActorWithMessageHandler};
use simple_actor::{SimpleActor, SimpleActorHandler};

#[tokio::main]
async fn main() {
    // Example #1: A very simple actor with an infinite loop that increments a state variable
    let simple_actor = SimpleActor::new("SimpleActor".to_string());
    let simple_actor_handler = SimpleActorHandler::new();
    simple_actor_handler.start(simple_actor);

    // Example 2: An actor with an infinite loop that listen for messages
    let (sender, receiver) = tokio::sync::mpsc::channel(100);
    let actor_message = ActorWithMessage::new("ActorWithMessage".to_string(), receiver);
    let actor_message_handler = ActorWithMessageHandler::new(sender);
    actor_message_handler.start(actor_message);
    start_seding_to_actor_message(&actor_message_handler);

    loop {}
}

fn start_seding_to_actor_message(handler: &ActorWithMessageHandler) {
    let handler = handler.clone();
    tokio::spawn(async move {
        let mut counter = 0;
        loop {
            handler
                .send_message(ActorMessage::Get { value: counter })
                .await;
            counter += 1;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
}
