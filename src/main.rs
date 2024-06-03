pub mod actor;
pub mod websocket_retry_actor;

use actor::websocket_actor::WebsocketActorHandler;
use tokio_util::sync::CancellationToken;
use websocket_retry_actor::{WebSocketRetryActor, WebSocketRetryActorHandler};

#[tokio::main]
async fn main() {
    // Example #1: A very simple actor with an infinite loop that increments its state variable
    let token = CancellationToken::new();
    let clone_token = token.clone();
    let (sender, receiver) = tokio::sync::mpsc::channel(100);
    let actor = WebSocketRetryActor::new("SimpleActor".to_string(), receiver, token);
    let actor_handler = WebSocketRetryActorHandler::new(sender, clone_token);
    actor_handler.start(actor);
    start_printing(&actor_handler);

    let mut counter = 0;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    loop {
        println!();
        println!("Main loop iteration: {}", counter);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        counter += 1;
        match counter {
            20 => {
                actor_handler.stop();
            }
            50 => {
                break;
            }
            _ => (),
        }
    }
}

fn start_printing(handler: &WebSocketRetryActorHandler) {
    let handler = handler.clone();
    tokio::spawn(async move {
        loop {
            let result = handler.get_data().await;

            println!("***** data received from actor = {:?}", result);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
}
