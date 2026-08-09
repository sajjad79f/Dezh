use contracts::Event;

use crate::{
    error::DefResult,
    registry::Middleware,
};

pub struct MetricsMiddleware;

impl Middleware for MetricsMiddleware {

    fn handle(

        &self,

        _event: &mut Event,

    ) -> DefResult<()> {

        Ok(())
    }
}