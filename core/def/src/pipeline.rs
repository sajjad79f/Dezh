use crate::{
    context::DefContext,
    models::EventContext,
};

pub trait PipelineStage: Send + Sync {

    fn name(&self) -> &'static str;

    fn execute(

        &self,

        def: &mut DefContext,

        event: &mut EventContext,
    );
}

pub struct EventPipeline {

    stages: Vec<Box<dyn PipelineStage>>,
}

impl EventPipeline {

    pub fn new() -> Self {

        Self {

            stages: Vec::new(),
        }
    }

    pub fn add<S>(
        &mut self,
        stage: S,
    )

    where
        S: PipelineStage + 'static,
    {

        self.stages.push(
            Box::new(stage),
        );
    }

    pub fn execute(

        &self,

        def: &mut DefContext,

        event: &mut EventContext,
    ) {

        for stage in &self.stages {

            if event.cancelled {

                break;
            }

            stage.execute(
                def,
                event,
            );
        }
    }
}