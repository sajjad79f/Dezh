use crate::models::EventContext;

pub struct EventValidator;

impl EventValidator {

    pub fn validate(

        context: &EventContext,
    ) -> bool {

        !context.event.topic.trim().is_empty()
    }
}