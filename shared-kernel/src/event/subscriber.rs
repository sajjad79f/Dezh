use contracts::{Event, EventHandler};

pub trait EventSubscriber {

    fn subscribe(
        &mut self,
        topic: &'static str,
        handler: Box<dyn EventHandler>,
    );

    fn notify(
        &self,
        event: &Event,
    );
}