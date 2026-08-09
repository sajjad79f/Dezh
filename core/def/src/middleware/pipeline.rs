use contracts::Event;

use crate::{
    error::DefResult,
    registry::MiddlewareRegistry,
};

pub struct MiddlewarePipeline<'a> {

    registry: &'a MiddlewareRegistry,
}

impl<'a> MiddlewarePipeline<'a> {

    pub fn new(

        registry: &'a MiddlewareRegistry,

    ) -> Self {

        Self {

            registry,
        }
    }

    pub fn execute(

        &self,

        event: &mut Event,

    ) -> DefResult<()> {

        for middleware in self.registry.all() {

            middleware.handle(event)?;
        }

        Ok(())
    }
}