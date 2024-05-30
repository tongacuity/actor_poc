pub mod actor;
pub mod simple_actor;

use actor::ActorHandler;
use simple_actor::SimpleActor;

#[tokio::main]
async fn main() {
    let simple_actor = SimpleActor::new("SimpleActor".to_string());
    let simple_actor_handler = simple_actor::SimpleActorHandler::new();
    simple_actor_handler.start(simple_actor);

    loop {}
}
