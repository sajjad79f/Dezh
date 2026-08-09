use crate::Event;

pub trait EventHandler: Send + Sync {
    fn handle(&self, event: &Event);
}