use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::{Rc, Weak};

pub type EventHandlerId = u32;

pub trait EventHandler<E> {
    fn handle(&mut self, event: &E);
}

enum Handler<E> {
    Rc(Rc<RefCell<dyn EventHandler<E>>>),
    Weak(Weak<RefCell<dyn EventHandler<E>>>),
}

pub struct EventHandlerManager<E> {
    next_id: EventHandlerId,
    handlers: HashMap<EventHandlerId, Handler<E>>,
}

impl<E> EventHandlerManager<E> {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            handlers: HashMap::new(),
        }
    }

    pub fn add(&mut self, handler: Rc<RefCell<dyn EventHandler<E>>>) -> EventHandlerId {
        let id = self.next_id;
        self.next_id += 1;
        self.handlers.insert(id, Handler::Rc(handler));
        id
    }

    pub fn add_weak(&mut self, handler: Weak<RefCell<dyn EventHandler<E>>>) -> EventHandlerId {
        let id = self.next_id;
        self.next_id += 1;
        self.handlers.insert(id, Handler::Weak(handler));
        id
    }

    pub fn remove_handler(&mut self, id: EventHandlerId) -> bool {
        self.handlers.remove(&id).is_some()
    }
}

impl<E> EventHandler<E> for EventHandlerManager<E> {
    fn handle(&mut self, event: &E) {
        let mut inactive_ids: Vec<EventHandlerId> = Vec::new();
        for (id, h) in &mut self.handlers {
            match h {
                Handler::Rc(h) => (*h.borrow_mut()).handle(&event),
                Handler::Weak(h) => {
                    if let Some(h) = h.upgrade() {
                        (*h.borrow_mut()).handle(&event);
                    } else {
                        inactive_ids.push(*id);
                    }
                }
            }
        }
        for id in inactive_ids {
            self.handlers.remove(&id);
        }
    }
}

impl<G> fmt::Debug for EventHandlerManager<G> {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum MyEvent {
        Hello,
        Say(&'static str),
    }

    struct MyEventHandler {
        name: &'static str,
        s: String,
    }

    impl EventHandler<MyEvent> for MyEventHandler {
        fn handle(&mut self, event: &MyEvent) {
            match event {
                MyEvent::Hello => {
                    self.s = format!("Hello, {}!", self.name);
                }
                MyEvent::Say(msg) => {
                    self.s = format!("{}: {}", self.name, msg);
                }
            }
        }
    }

    #[test]
    fn basic() {
        let mut mgr: EventHandlerManager<MyEvent> = EventHandlerManager::new();
        let handler = Rc::new(RefCell::new(MyEventHandler {
            name: "Alice",
            s: String::default(),
        }));
        let id = mgr.add(handler.clone());
        mgr.handle(&MyEvent::Hello);
        assert_eq!("Hello, Alice!", handler.borrow().s);
        handler.borrow_mut().name = "Bob";
        mgr.handle(&MyEvent::Say("foo"));
        assert_eq!("Bob: foo", handler.borrow().s);
        mgr.remove_handler(id);
        mgr.handle(&MyEvent::Hello);
        assert_eq!("Bob: foo", handler.borrow().s);
    }
}
