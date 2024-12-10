use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::any::Any;

trait Message: Send + 'static {}
impl<T: Send + 'static> Message for T {}

trait Actor: Send + 'static {
    fn handle_message(&mut self, msg: Box<dyn Any + Send>);
}

struct ActorHandle<A: Actor> {
    sender: Sender<Box<dyn Any + Send>>,
    _phantom: std::marker::PhantomData<A>,
}

impl<A: Actor> ActorHandle<A> {
    pub fn send<M: Message>(&self, msg: M) -> Result<(), String> {
        self.sender
            .send(Box::new(msg))
            .map_err(|_| "Actor has terminated".to_string())
    }
}

struct ActorSystem {
    actors: Vec<ActorHandle<Box<dyn Actor>>>,
}

impl ActorSystem {
    pub fn new() -> Self {
        ActorSystem {
            actors: Vec::new(),
        }
    }

    pub fn spawn<A: Actor>(&mut self, mut actor: A) -> ActorHandle<A> {
        let (tx, rx) = channel();
        
        let handle = ActorHandle {
            sender: tx,
            _phantom: std::marker::PhantomData,
        };
        
        thread::spawn(move || {
            Self::actor_loop(&mut actor, rx);
        });
        
        handle
    }

    fn actor_loop<A: Actor>(actor: &mut A, receiver: Receiver<Box<dyn Any + Send>>) {
        while let Ok(message) = receiver.recv() {
            actor.handle_message(message);
        }
    }
}
