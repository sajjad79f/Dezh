use crate::{
    context::DefContext,
    dispatcher::EventDispatcher,
    models::EventContext,
    pipeline::PipelineStage,
};

pub struct DispatchStage;

impl PipelineStage for DispatchStage {

    fn name(&self) -> &'static str {

        "dispatch"
    }

    fn execute(

        &self,

        def: &mut DefContext,

        event: &mut EventContext,
    ) {

        EventDispatcher::dispatch(

            def.registry(),

            event,
        );

        def.metrics().dispatched();
    }
}