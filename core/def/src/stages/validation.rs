use crate::{
    context::DefContext,
    models::EventContext,
    pipeline::PipelineStage,
    validator::EventValidator,
};

pub struct ValidationStage;

impl PipelineStage for ValidationStage {

    fn name(&self) -> &'static str {

        "validation"
    }

    fn execute(

        &self,

        _: &mut DefContext,

        event: &mut EventContext,
    ) {

        if !EventValidator::validate(event) {

            event.cancel();
        }
    }
}