use crate::{
    context::DefContext,
    models::EventContext,
    pipeline::PipelineStage,
};

pub struct RoutingStage;

impl PipelineStage for RoutingStage {

    fn name(&self) -> &'static str {

        "routing"
    }

    fn execute(

        &self,

        _: &mut DefContext,

        event: &mut EventContext,
    ) {

        event.insert(
            "route",
            event.event.topic.clone(),
        );
    }
}