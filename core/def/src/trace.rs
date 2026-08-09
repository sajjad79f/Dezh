use crate::models::EventContext;

pub struct EventTracer;

impl EventTracer {

    pub fn trace(

        context: &EventContext,
    ) {

        eprintln!(
            "[TRACE] {}",
            context.event.topic,
        );
    }
}