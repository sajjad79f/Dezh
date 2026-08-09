use crate::{
    context::DefContext,
    models::EventContext,
    pipeline::PipelineStage,
    trace::EventTracer,
};

pub struct TraceStage;

impl PipelineStage for TraceStage {

    fn name(&self) -> &'static str {

        "trace"
    }

    fn execute(

        &self,

        _: &mut DefContext,

        event: &mut EventContext,
    ) {

        EventTracer::trace(event);
    }
}