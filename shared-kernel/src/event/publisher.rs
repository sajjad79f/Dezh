use contracts::Event;

pub trait EventPublisher {

    fn publish(&self, event: Event);

}