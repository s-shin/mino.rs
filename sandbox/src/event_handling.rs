use std::any::Any;
use std::collections::HashMap;

trait EventHandler: Any {
    fn hello(&mut self) {}
    fn say(&mut self, _msg: &str) {}
    fn as_any(&self) -> &dyn Any;
}

type EventHandlerId = u32;

#[derive(Default)]
struct EventHandlerManager {
    last_id: EventHandlerId,
    handlers: HashMap<EventHandlerId, Box<dyn EventHandler>>,
}

impl EventHandlerManager {
    fn add(&mut self, handler: Box<dyn EventHandler>) -> EventHandlerId {
        self.last_id += 1;
        self.handlers.insert(self.last_id, handler);
        self.last_id
    }
    fn remove(&mut self, id: EventHandlerId) -> Option<Box<dyn EventHandler>> {
        self.handlers.remove(&id)
    }
    fn get(&self, id: EventHandlerId) -> Option<&Box<dyn EventHandler>> {
        self.handlers.get(&id)
    }
}

impl EventHandler for EventHandlerManager {
    fn hello(&mut self) {
        for (_, handler) in &mut self.handlers {
            handler.hello();
        }
    }
    fn say(&mut self, msg: &str) {
        for (_, handler) in &mut self.handlers {
            handler.say(msg);
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct SomeoneHandler {
    name: &'static str,
    hello_count: u8,
}

impl SomeoneHandler {
    fn new(name: &'static str) -> Self {
        Self {
            name: name,
            hello_count: 0,
        }
    }
}

impl EventHandler for SomeoneHandler {
    fn hello(&mut self) {
        println!("Hello, {}!", self.name);
        self.hello_count += 1;
    }
    fn say(&mut self, msg: &str) {
        println!("{}: {}", self.name, msg);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn run() {
    let mut mgr = EventHandlerManager::default();
    let id_alice = mgr.add(Box::new(SomeoneHandler::new("Alice")));
    let id_bob = mgr.add(Box::new(SomeoneHandler::new("Bob")));
    mgr.hello();
    mgr.say("hi");
    mgr.remove(id_alice);
    mgr.hello();
    if let Some(handler) = mgr
        .get(id_bob)
        .unwrap()
        .as_any()
        .downcast_ref::<SomeoneHandler>()
    {
        println!("Bob hello_count: {}", handler.hello_count);
    }
}
