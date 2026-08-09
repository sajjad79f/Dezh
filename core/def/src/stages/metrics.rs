use crate::{
    context::DefContext,
    models::EventContext,
    pipeline::PipelineStage,
};

pub struct MetricsStage;

impl PipelineStage for MetricsStage {

    fn name(&self) -> &'static str {

        "metrics"
    }

    fn execute(

        &self,

        def: &mut DefContext,

        _: &mut EventContext,
    ) {

        def.metrics().published();
    }
}