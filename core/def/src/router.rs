use crate::{
    dispatcher::EventDispatcher,
    models::EventContext,
    subscription_registry::SubscriptionRegistry,
};

pub struct EventRouter;

impl EventRouter {

    pub fn route(

        registry: &SubscriptionRegistry,

        context: &mut EventContext,
    ) {

        EventDispatcher::dispatch(

            registry,

            context,
        );
    }
}