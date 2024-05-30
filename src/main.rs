pub mod actor;
pub mod actor_with_message;
pub mod simple_actor;
pub mod websocket_actor;

use actor::ActorHandler;
use actor_with_message::{ActorMessage, ActorWithMessage, ActorWithMessageHandler};
use simple_actor::{SimpleActor, SimpleActorHandler};
use websocket_actor::{WebSocketActor, WebSocketActorHandler};

#[tokio::main]
async fn main() {
    // Example #1: A very simple actor with an infinite loop that increments its state variable
    let simple_actor = SimpleActor::new("SimpleActor".to_string());
    let simple_actor_handler = SimpleActorHandler::new();
    simple_actor_handler.start(simple_actor);

    // Example 2: An actor with an infinite loop that listen for messages sent from main.rs
    let (sender, receiver) = tokio::sync::mpsc::channel(100);
    let actor_message = ActorWithMessage::new("ActorWithMessage".to_string(), receiver);
    let actor_message_handler = ActorWithMessageHandler::new(sender);
    actor_message_handler.start(actor_message);
    start_seding_to_actor_message(&actor_message_handler);

    // Example 3: An actor which is also a websocket client which connects to binance and listen on BBO update channel
    let (sender, receiver) = tokio::sync::mpsc::channel(100);
    let websocket_client = WebSocketActor::new("WebosocetActor".to_string(), receiver);
    let websocket_handler = WebSocketActorHandler::new(sender);
    websocket_handler.start(websocket_client);
    start_printing_bbo(&websocket_handler);

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    loop {        
        println!();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
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

fn start_printing_bbo(handler: &WebSocketActorHandler) {
    let handler: WebSocketActorHandler = handler.clone();
    tokio::spawn(async move {
        loop {
            let result = handler.get_bbo_data().await;

            println!("***** BBO data = {:?}", result);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
}
