use crate::{
    errors::DefError,
    models::EventContext,
    subscription_registry::SubscriptionRegistry,
};

pub struct EventDispatcher;

impl EventDispatcher {

    pub fn dispatch(

        registry: &SubscriptionRegistry,

        context: &mut EventContext,
    ) {

        if context.cancelled {

            return;
        }

        if let Some(handlers) = registry.handlers(
            &context.event.topic,
        ) {

            for handler in handlers {

                handler.handle(
                    &context.event,
                );
            }
        } else {

            let error = DefError::NoHandler(context.event.topic.clone());

            eprintln!("[DEF] {error}");
        }

        context.dispatched();
    }
}