use contracts::Event;

use crate::{
    error::DefResult,
    registry::Middleware,
    validation::EventValidator,
};

pub struct ValidationMiddleware;

impl Middleware for ValidationMiddleware {

    fn handle(

        &self,

        event: &mut Event,

    ) -> DefResult<()> {

        EventValidator::validate(event)
    }
}